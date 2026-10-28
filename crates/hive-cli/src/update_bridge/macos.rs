//! Authenticate the running predecessor, even after its on-disk image was replaced.
use super::{posix, LegacyBudget};
use crate::usage::{CommandRunner, SystemCommandRunner};
use core_foundation::{base::TCFType, boolean::CFBoolean, string::CFString};
use libproc::{
    bsd_info::BSDInfo,
    proc_pid::{pidinfo, pidpath},
};
use security_framework::os::macos::code_signing::{
    Flags, GuestAttributes, SecCode, SecRequirement,
};
use std::path::{Path, PathBuf};
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

const IMAGES: &str = include_str!("../../../../harness/setup/legacy-update-posix-images.json");

fn info(pid: u32) -> Result<BSDInfo, String> {
    pidinfo(i32::try_from(pid).map_err(|_| "invalid parent pid")?, 0)
        .map_err(|_| "parent process evidence unavailable".into())
}

pub(super) struct Parent {
    observed: BSDInfo,
}

impl Parent {
    pub(super) fn alive(&self) -> bool {
        let old = &self.observed;
        info(std::process::id()).is_ok_and(|child| child.pbi_ppid == old.pbi_pid)
            && info(old.pbi_pid).is_ok_and(|current| {
                current.pbi_status != 5 // SZOMB
                    && current.pbi_pid == old.pbi_pid
                    && current.pbi_uid == old.pbi_uid
                    && current.pbi_start_tvsec == old.pbi_start_tvsec
                    && current.pbi_start_tvusec == old.pbi_start_tvusec
            })
    }
}

// Refresh only this parent's command. No environment, task or other process details are requested.
fn bounded_arguments(values: &[std::ffi::OsString]) -> Result<Vec<String>, String> {
    if !(2..=16).contains(&values.len()) {
        return Err("unexpected legacy argument count".into());
    }
    values
        .iter()
        .map(|value| {
            let value = value.to_str().ok_or("parent argv is not UTF-8")?;
            if value.is_empty() || value.len() > 4096 {
                return Err("invalid parent argument".into());
            }
            Ok(value.to_owned())
        })
        .collect()
}

fn arguments(pid: u32) -> Result<Vec<String>, String> {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing()
            .without_tasks()
            .with_cmd(UpdateKind::Always),
    );
    let process = system.process(pid).ok_or("parent argv unavailable")?;
    if system.processes().len() != 1 || !process.environ().is_empty() {
        return Err("parent inspection scope differs".into());
    }
    bounded_arguments(process.cmd())
}

fn known_running_image(pid: u32) -> Result<bool, String> {
    let registry: serde_json::Value =
        serde_json::from_str(IMAGES).map_err(|_| "invalid parent registry")?;
    let platform = if cfg!(target_arch = "aarch64") {
        "darwin-arm64"
    } else {
        "unqualified"
    };
    let images = registry["images"]
        .as_array()
        .ok_or("missing parent registry")?
        .iter()
        .filter(|entry| {
            entry["platform"] == platform
                && matches!(
                    entry["version"].as_str(),
                    Some("0.13.1" | "0.13.2-test.1" | "0.13.2-test.2")
                )
        });
    let mut hashes = Vec::new();
    for image in images {
        let hash = image["code_directory_hash"]
            .as_str()
            .ok_or("missing parent code hash")?;
        if hash.len() != 40 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid parent code hash".into());
        }
        hashes.push(hash);
    }
    if hashes.is_empty() {
        return Ok(false);
    }
    let mut attributes = GuestAttributes::new();
    attributes.set_pid(i32::try_from(pid).map_err(|_| "invalid parent pid")?);
    let key = CFString::new("dynamicCode");
    attributes.set_other(
        key.as_concrete_TypeRef(),
        CFBoolean::true_value().as_CFTypeRef(),
    );
    // dynamicCode makes Security use the kernel's retained CodeDirectory, not the new file.
    let code = SecCode::copy_guest_with_attribues(None, &attributes, Flags::NONE)
        .map_err(|_| "running parent signature unavailable")?;
    for hash in hashes {
        let requirement: SecRequirement = format!("cdhash H\"{hash}\"")
            .parse()
            .map_err(|_| "invalid parent signature requirement")?;
        if code.check_validity(Flags::NONE, &requirement).is_ok() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn terminal_descriptors(bytes: &[u8], device: u32) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mut current = None;
    let mut observed = std::collections::BTreeMap::new();
    for field in text.split(['\0', '\n']).filter(|s| !s.is_empty()) {
        let (key, value) = field.split_at(1);
        match key {
            "f" => {
                current = value.parse::<u32>().ok().filter(|fd| matches!(fd, 0 | 2));
                let Some(fd) = current else {
                    return false;
                };
                if observed.insert(fd, (false, false)).is_some() {
                    return false;
                }
            }
            "t" => {
                if let Some(fd) = current {
                    observed.get_mut(&fd).unwrap().0 = value == "CHR";
                }
            }
            "r" => {
                if let Some(fd) = current {
                    observed.get_mut(&fd).unwrap().1 = value
                        .strip_prefix("0x")
                        .and_then(|s| u32::from_str_radix(s, 16).ok())
                        == Some(device);
                }
            }
            _ => {}
        }
    }
    observed.len() == 2 && observed.values().all(|pair| *pair == (true, true))
}

fn terminal(parent: &BSDInfo, child: &BSDInfo) -> bool {
    if matches!(parent.e_tdev, 0 | u32::MAX)
        || parent.e_tdev != child.e_tdev
        || parent.pbi_pgid != parent.e_tpgid
        || child.pbi_pgid != child.e_tpgid
    {
        return false;
    }
    let runner = SystemCommandRunner;
    let Ok(program) = runner.qualify("/usr/sbin/lsof") else {
        return false;
    };
    let pid = parent.pbi_pid.to_string();
    runner
        .run(
            &program,
            &["-nP", "-a", "-p", &pid, "-d", "0,2", "-F0ftr"],
            Duration::from_secs(2),
            8192,
        )
        .is_ok_and(|result| result.success && terminal_descriptors(&result.stdout, parent.e_tdev))
}

pub(super) fn connect(
    root: &Path,
    budget: LegacyBudget,
) -> Result<Option<posix::Connection>, String> {
    let child = info(std::process::id())?;
    let parent = Parent {
        observed: info(child.pbi_ppid)?,
    };
    let old = &parent.observed;
    let path = PathBuf::from(
        pidpath(i32::try_from(old.pbi_pid).map_err(|_| "invalid parent pid")?)
            .map_err(|_| "parent executable unavailable")?,
    );
    if path.file_name().and_then(|s| s.to_str()) != Some("hive") {
        return Ok(None);
    }
    if old.pbi_uid != child.pbi_uid || old.pbi_ruid != child.pbi_ruid {
        return Err("legacy parent owner differs".into());
    }
    let argv = arguments(old.pbi_pid)?;
    if argv.get(1).map(String::as_str) != Some("update") {
        return Ok(None);
    }
    if !known_running_image(old.pbi_pid)? {
        return Ok(None);
    }
    let current = std::env::current_exe().map_err(|_| "current executable unavailable")?;
    if path
        .canonicalize()
        .map_err(|_| "legacy install path unavailable")?
        != current
            .canonicalize()
            .map_err(|_| "current install path unavailable")?
    {
        return Err("legacy updater installation scope differs".into());
    }
    let needs_terminal = posix::interactive(&argv, root)? && terminal(old, &child);
    if !parent.alive() {
        return Err("legacy parent changed".into());
    }
    posix::connect(parent, budget, needs_terminal).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argv_preserves_spaces_and_rejects_unbounded_or_empty_arguments() {
        let args =
            ["hive", "update", "--user-root", "/tmp/user name"].map(std::ffi::OsString::from);
        assert_eq!(
            bounded_arguments(&args).unwrap(),
            ["hive", "update", "--user-root", "/tmp/user name"]
        );
        assert!(bounded_arguments(&vec![std::ffi::OsString::from("x"); 17]).is_err());
        assert!(bounded_arguments(&["hive".into(), "".into()]).is_err());
        assert!(bounded_arguments(&["hive".into(), "x".repeat(4097).into()]).is_err());
    }

    #[test]
    fn terminal_requires_both_exact_character_device_descriptors() {
        let good = b"p123\0\nf0\0tCHR\0r0x04000001\0\nf2\0tCHR\0r0x04000001\0\n";
        assert!(terminal_descriptors(good, 0x0400_0001));
        assert!(!terminal_descriptors(good, 0x0400_0002));
        assert!(!terminal_descriptors(
            b"f0\0tCHR\0r0x04000001\0",
            0x0400_0001
        ));
        assert!(!terminal_descriptors(
            b"f0\0tREG\0r0x04000001\0f2\0tCHR\0r0x04000001\0",
            0x0400_0001
        ));
    }

    #[test]
    fn native_self_arguments_and_unqualified_image_are_observable() {
        let pid = std::process::id();
        assert_eq!(info(pid).unwrap().pbi_pid, pid);
        // The unfiltered test runner has only argv[0], which is correctly not a legacy update.
        let expected = std::env::args_os().collect::<Vec<_>>();
        assert_eq!(arguments(pid), bounded_arguments(&expected));
        assert!(!known_running_image(pid).unwrap());
    }
}
