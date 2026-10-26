//! Authenticate the retained Linux parent image, including an unlinked npm predecessor.
use super::{posix, LegacyBudget};
use posix::Connection;
use std::fs::File;
use std::io::{IsTerminal, Read};
use std::os::fd::AsRawFd;
use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
use std::path::{Path, PathBuf};

const IMAGES: &str = include_str!("../../../../harness/setup/legacy-update-posix-images.json");

#[derive(Clone, Debug, Eq, PartialEq)]
struct Stat {
    parent: u32,
    group: i64,
    session: i64,
    tty: i64,
    foreground: i64,
    started: u64,
    zombie: bool,
}

fn stat(bytes: &[u8]) -> Result<Stat, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid parent stat encoding")?;
    let (_, fields) = text.rsplit_once(") ").ok_or("invalid parent stat")?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    let number = |index: usize| -> Result<i64, String> {
        fields
            .get(index)
            .ok_or_else(|| "incomplete parent stat".to_owned())?
            .parse()
            .map_err(|_| "invalid parent stat field".to_owned())
    };
    Ok(Stat {
        parent: u32::try_from(number(1)?).map_err(|_| "invalid parent pid")?,
        group: number(2)?,
        session: number(3)?,
        tty: number(4)?,
        foreground: number(5)?,
        started: u64::try_from(number(19)?).map_err(|_| "invalid parent birth")?,
        zombie: matches!(fields.first().copied(), Some("Z" | "X")),
    })
}

fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|_| "cannot open parent evidence")?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read parent evidence")?;
    if bytes.len() as u64 > limit {
        return Err("parent evidence exceeds limit".into());
    }
    Ok(bytes)
}

pub(super) struct Parent {
    directory: File,
    observed: Stat,
}
impl Parent {
    fn path(&self, relative: &str) -> PathBuf {
        PathBuf::from(format!(
            "/proc/self/fd/{}/{relative}",
            self.directory.as_raw_fd()
        ))
    }
    pub(super) fn alive(&self) -> bool {
        bounded(&self.path("stat"), 8192)
            .and_then(|b| stat(&b))
            .is_ok_and(|current| !current.zombie && current.started == self.observed.started)
    }
}

#[allow(clippy::too_many_lines)]
pub(super) fn connect(root: &Path, budget: LegacyBudget) -> Result<Option<Connection>, String> {
    let child = stat(&bounded(Path::new("/proc/self/stat"), 8192)?)?;
    let directory =
        File::open(format!("/proc/{}", child.parent)).map_err(|_| "parent disappeared")?;
    let mut parent = Parent {
        directory,
        observed: child.clone(),
    };
    parent.observed = stat(&bounded(&parent.path("stat"), 8192)?)?;
    let arguments = bounded(&parent.path("cmdline"), 65536)?;
    let arguments: Vec<String> = arguments
        .split(|byte| *byte == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8(s.to_vec()).map_err(|_| "legacy argv is not UTF-8".to_owned()))
        .collect::<Result<_, _>>()?;
    if arguments.get(1).map(String::as_str) != Some("update") {
        return Ok(None);
    }
    let name =
        std::fs::read_link(parent.path("exe")).map_err(|_| "parent executable unavailable")?;
    let raw = name.as_os_str().as_bytes();
    let name = Path::new(std::ffi::OsStr::from_bytes(
        raw.strip_suffix(b" (deleted)").unwrap_or(raw),
    ));
    if name.file_name().and_then(|n| n.to_str()) != Some("hive") {
        return Ok(None);
    }
    let metadata = parent
        .directory
        .metadata()
        .map_err(|_| "parent owner unavailable")?;
    if metadata.uid()
        != std::fs::metadata("/proc/self")
            .map_err(|_| "caller owner unavailable")?
            .uid()
    {
        return Err("legacy parent owner differs".into());
    }
    let image = bounded(&parent.path("exe"), 128 * 1024 * 1024)?;
    let digest = hive_core::sha256_digest(&image);
    let registry: serde_json::Value =
        serde_json::from_str(IMAGES).map_err(|_| "invalid parent registry")?;
    let platform = if cfg!(target_arch = "x86_64") {
        "linux-x64"
    } else {
        "unqualified"
    };
    let known = registry["images"]
        .as_array()
        .ok_or("missing parent registry")?
        .iter()
        .any(|entry| {
            entry["platform"] == platform
                && entry["version"] == "0.13.1"
                && entry["binary_sha256"].as_str() == digest.strip_prefix("sha256:")
        });
    if !known {
        return Ok(None);
    }
    let current = std::env::current_exe().map_err(|_| "current executable unavailable")?;
    if name
        .canonicalize()
        .map_err(|_| "legacy install path unavailable")?
        != current
            .canonicalize()
            .map_err(|_| "current install path unavailable")?
    {
        return Err("legacy updater installation scope differs".into());
    }
    let needs_terminal = posix::interactive(&arguments, root)?;
    let prior = &parent.observed;
    if needs_terminal
        && (child.tty == 0
            || prior.tty != child.tty
            || prior.session != child.session
            || prior.group != prior.foreground
            || child.group != child.foreground)
    {
        return Err("legacy controlling terminal cannot be proved".into());
    }
    for fd in ["fd/0", "fd/2"].into_iter().filter(|_| needs_terminal) {
        if !File::open(parent.path(fd))
            .map_err(|_| "legacy terminal descriptor unavailable")?
            .is_terminal()
        {
            return Err("legacy terminal descriptors differ".into());
        }
    }
    posix::connect(parent, budget, needs_terminal).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_proc_stat_and_owned_directory_are_readable() {
        let current = stat(&bounded(Path::new("/proc/self/stat"), 8192).unwrap()).unwrap();
        assert!(current.parent > 0 && current.started > 0 && !current.zombie);
        let parent = Parent {
            directory: File::open("/proc/self").unwrap(),
            observed: current,
        };
        assert!(parent.alive());
    }
}
