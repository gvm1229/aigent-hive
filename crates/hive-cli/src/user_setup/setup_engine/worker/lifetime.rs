//! A killed coordinator must not leave its detached apply group running.
use crate::user_setup::SetupError;
use nix::{
    sys::signal::{killpg, Signal},
    unistd::{getpgrp, getppid, Pid},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub(super) struct OwnedGroup(Arc<AtomicBool>);

impl OwnedGroup {
    pub fn start(parent_process_id: u32) -> Result<Self, SetupError> {
        let own = Pid::this();
        if getpgrp() != own {
            return Err(SetupError::Conflict(
                "setup worker requires its own process group".into(),
            ));
        }
        let parent = i32::try_from(parent_process_id)
            .ok()
            .filter(|pid| *pid > 0)
            .map(Pid::from_raw)
            .ok_or_else(|| SetupError::Input("setup worker parent is missing".into()))?;
        if getppid() != parent {
            return Err(SetupError::Conflict(
                "setup coordinator ended before worker activation".into(),
            ));
        }
        let active = Arc::new(AtomicBool::new(true));
        let watching = Arc::clone(&active);
        thread::Builder::new()
            .name("hive-setup-lifetime".into())
            .spawn(move || {
                while watching.load(Ordering::Acquire) {
                    if getppid() != parent {
                        // Never signal a group if its ownership changed underneath us.
                        if getpgrp() == own {
                            let _ = killpg(own, Signal::SIGKILL);
                        }
                        std::process::exit(2);
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            })
            .map_err(|_| SetupError::Internal("cannot bind setup worker lifetime".into()))?;
        Ok(Self(active))
    }
}

impl Drop for OwnedGroup {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
