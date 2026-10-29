//! Authenticate the running predecessor, even after its on-disk image was replaced.
use super::{posix, LegacyBudget};
use crate::usage::{CommandRunner, SystemCommandRunner};
use base64::Engine;
use core_foundation::{base::TCFType, boolean::CFBoolean, string::CFString};
use libproc::{
    bsd_info::BSDInfo,
    proc_pid::{pidinfo, pidpath},
};
use security_framework::os::macos::code_signing::{
    Flags, GuestAttributes, SecCode, SecRequirement,
};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

const IMAGES: &str = include_str!("../../../../harness/setup/legacy-update-posix-images.json");
const KERNEL_IDENTITY: &str = include_str!("macos-kernel-identity.js");

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

// Refresh only this parent's command and kernel exec path. Never request its environment or tasks.
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

fn command(pid: u32) -> Result<(PathBuf, Vec<String>), String> {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing()
            .without_tasks()
            .with_cmd(UpdateKind::Always)
            .with_exe(UpdateKind::Always),
    );
    let process = system.process(pid).ok_or("parent argv unavailable")?;
    if system.processes().len() != 1 || !process.environ().is_empty() {
        return Err("parent inspection scope differs".into());
    }
    let executable = process.exe().ok_or("parent kernel exec path unavailable")?;
    if !executable.is_absolute() {
        return Err("parent kernel exec path is not absolute".into());
    }
    Ok((executable.to_path_buf(), bounded_arguments(process.cmd())?))
}

fn known_image_hashes() -> Result<Vec<String>, String> {
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
        hashes.push(hash.to_owned());
    }
    Ok(hashes)
}

fn known_running_image(pid: u32) -> Result<bool, String> {
    let hashes = known_image_hashes()?;
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

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct KernelIdentity {
    pid: u32,
    flags: u32,
    code_directory_hash: String,
}

fn kernel_image_matches(bytes: &[u8], pid: u32, hashes: &[String]) -> Result<bool, String> {
    let identity: KernelIdentity =
        serde_json::from_slice(bytes).map_err(|_| "invalid parent kernel identity")?;
    if identity.pid != pid || identity.flags & 1 == 0 {
        // XNU CS_VALID
        return Ok(false);
    }
    let digest = base64::engine::general_purpose::STANDARD
        .decode(identity.code_directory_hash)
        .map_err(|_| "invalid parent code hash")?;
    if digest.len() != 20 {
        return Err("invalid parent code hash length".into());
    }
    let mut actual = String::with_capacity(40);
    for byte in digest {
        write!(actual, "{byte:02x}").expect("formatting into a String cannot fail");
    }
    Ok(hashes
        .iter()
        .any(|expected| expected.eq_ignore_ascii_case(&actual)))
}

fn known_unlinked_image(pid: u32) -> Result<bool, String> {
    let hashes = known_image_hashes()?;
    if hashes.is_empty() {
        return Ok(false);
    }
    let runner = SystemCommandRunner;
    let program = runner
        .qualify("/usr/bin/osascript")
        .map_err(|_| "kernel identity reader unavailable")?;
    let result = runner
        .run(
            &program,
            &[
                "-l",
                "JavaScript",
                "-e",
                KERNEL_IDENTITY,
                "--",
                &pid.to_string(),
            ],
            Duration::from_secs(2),
            4096,
        )
        .map_err(|_| "parent kernel identity unavailable")?;
    if !result.success {
        return Err("parent kernel identity reader failed".into());
    }
    kernel_image_matches(&result.stdout, pid, &hashes)
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
    let native_path = pidpath(i32::try_from(old.pbi_pid).map_err(|_| "invalid parent pid")?)
        .ok()
        .map(PathBuf::from);
    if native_path
        .as_ref()
        .is_some_and(|path| path.file_name().and_then(|s| s.to_str()) != Some("hive"))
    {
        return Ok(None);
    }
    if old.pbi_uid != child.pbi_uid || old.pbi_ruid != child.pbi_ruid {
        return Err("legacy parent owner differs".into());
    }
    let (kernel_path, argv) = command(old.pbi_pid)?;
    let path = native_path.as_ref().unwrap_or(&kernel_path);
    if path.file_name().and_then(|s| s.to_str()) != Some("hive") {
        return Ok(None);
    }
    if argv.get(1).map(String::as_str) != Some("update") {
        return Ok(None);
    }
    let current = std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .map_err(|_| "current executable unavailable")?;
    if native_path.is_none() {
        // npm can unlink the old image's entire directory. The kernel retains its
        // original exec path and CodeDirectory hash; the replacement file is not identity evidence.
        if !crate::update_activation::is_npm_installation(&current)
            || kernel_path.canonicalize().ok().as_ref() != Some(&current)
            || Path::new(&argv[0]).canonicalize().ok().as_ref() != Some(&current)
        {
            return Err("unlinked legacy updater installation scope differs".into());
        }
        if !known_unlinked_image(old.pbi_pid)? {
            return Ok(None);
        }
    } else if !known_running_image(old.pbi_pid)? {
        return Ok(None);
    }
    if !super::same_installation_path(path, &argv[0], &current)? {
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
    fn kernel_identity_requires_exact_pid_valid_code_and_registered_hash() {
        let hash = base64::engine::general_purpose::STANDARD.encode([1_u8; 20]);
        let hashes = vec!["01".repeat(20)];
        let good = serde_json::json!({"pid": 7, "flags": 1, "code_directory_hash": hash});
        let check = |value: &serde_json::Value, pid, hashes: &[String]| {
            kernel_image_matches(&serde_json::to_vec(value).unwrap(), pid, hashes)
        };
        assert!(check(&good, 7, &hashes).unwrap());
        assert!(!check(&good, 8, &hashes).unwrap());
        assert!(!check(&good, 7, &["02".repeat(20)]).unwrap());
        let mut invalid = good.clone();
        invalid["flags"] = 0.into();
        assert!(!check(&invalid, 7, &hashes).unwrap());
        invalid = good.clone();
        invalid["code_directory_hash"] = "not-base64".into();
        assert!(check(&invalid, 7, &hashes).is_err());
        invalid["code_directory_hash"] = base64::engine::general_purpose::STANDARD
            .encode([1_u8; 19])
            .into();
        assert!(check(&invalid, 7, &hashes).is_err());
        invalid = good.clone();
        invalid["extra"] = true.into();
        assert!(check(&invalid, 7, &hashes).is_err());
        assert!(kernel_image_matches(b"{}", 7, &hashes).is_err());
    }

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
        assert_eq!(
            command(pid).map(|(_, argv)| argv),
            bounded_arguments(&expected)
        );
        assert!(!known_running_image(pid).unwrap());
        assert!(!known_unlinked_image(pid).unwrap());
    }
}
