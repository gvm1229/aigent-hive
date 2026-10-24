//! Authenticate the retained Linux parent image, including an unlinked npm predecessor.
use super::LegacyBudget;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, IsTerminal, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

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

struct Parent {
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
    fn alive(&self) -> bool {
        bounded(&self.path("stat"), 8192)
            .and_then(|b| stat(&b))
            .is_ok_and(|current| !current.zombie && current.started == self.observed.started)
    }
}

pub(super) enum Connection {
    Terminal(Reader, Writer),
    Preserve(Arc<Watch>),
}
pub(super) struct Watch {
    parent: Parent,
    budget: LegacyBudget,
}
impl Watch {
    fn check(&self) -> io::Result<()> {
        if !self.budget.is_connected() || !self.parent.alive() {
            self.budget.disconnect();
            return Err(io::Error::other("legacy updater disconnected"));
        }
        Ok(())
    }
    fn wait<T>(&self, receive: &mpsc::Receiver<T>, until: Instant) -> io::Result<T> {
        loop {
            self.check()?;
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "legacy terminal deadline",
                ));
            }
            match receive.recv_timeout(left.min(Duration::from_millis(100))) {
                Ok(value) => return Ok(value),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(io::Error::other("legacy terminal ended"))
                }
            }
        }
    }
}

fn interactive(arguments: &[String], root: &Path) -> Result<bool, String> {
    let mut supplied = None;
    let mut confirm = false;
    let mut seen = std::collections::BTreeSet::new();
    let mut index = 2;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if !seen.insert(option) {
            return Err("duplicate legacy option".into());
        }
        if option == "--confirm" {
            confirm = true;
            index += 1;
            continue;
        }
        let value = arguments
            .get(index + 1)
            .ok_or("missing legacy option value")?;
        match option {
            "--user-root" => supplied = Some(PathBuf::from(value)),
            "--channel" if matches!(value.as_str(), "stable" | "test") => {}
            _ => return Err("unknown legacy option".into()),
        }
        index += 2;
    }
    let supplied = supplied.map_or_else(crate::user_install::resolve_user_root_path, Ok)?;
    if supplied
        .canonicalize()
        .map_err(|_| "legacy user scope unavailable")?
        != root
            .canonicalize()
            .map_err(|_| "current user scope unavailable")?
    {
        return Err("legacy updater user scope differs".into());
    }
    Ok(!confirm)
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
    let needs_terminal = interactive(&arguments, root)?;
    if !parent.alive() {
        return Err("legacy parent changed".into());
    }
    let watch = Arc::new(Watch { parent, budget });
    let weak = Arc::downgrade(&watch);
    thread::spawn(move || loop {
        let Some(watch) = weak.upgrade() else { break };
        if watch.check().is_err() {
            break;
        }
        drop(watch);
        thread::sleep(Duration::from_millis(100));
    });
    if !needs_terminal {
        return Ok(Some(Connection::Preserve(watch)));
    }
    let prior = &watch.parent.observed;
    if child.tty == 0
        || prior.tty != child.tty
        || prior.session != child.session
        || prior.group != prior.foreground
        || child.group != child.foreground
    {
        return Err("legacy controlling terminal cannot be proved".into());
    }
    for fd in ["fd/0", "fd/2"] {
        if !File::open(watch.parent.path(fd))
            .map_err(|_| "legacy terminal descriptor unavailable")?
            .is_terminal()
        {
            return Err("legacy terminal descriptors differ".into());
        }
    }
    let input = File::open("/dev/tty").map_err(|_| "legacy controlling terminal unavailable")?;
    let mut output = OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .map_err(|_| "legacy terminal output unavailable")?;
    let (send, receive) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut input = BufReader::new(input);
        loop {
            let mut bytes = Vec::new();
            let result = input.by_ref().take(4097).read_until(b'\n', &mut bytes);
            let ended = result.as_ref().is_ok_and(|count| *count == 0);
            let value = result.and_then(|_| {
                if bytes.len() > 4096 {
                    Err(io::Error::other("legacy input limit"))
                } else {
                    Ok(bytes)
                }
            });
            if send.send(value).is_err() || ended {
                break;
            }
        }
    });
    let (write, writes) = mpsc::sync_channel::<(Vec<u8>, mpsc::Sender<io::Result<()>>)>(1);
    thread::spawn(move || {
        while let Ok((bytes, reply)) = writes.recv() {
            let result = output.write_all(&bytes).and_then(|()| output.flush());
            if reply.send(result).is_err() {
                break;
            }
        }
    });
    Ok(Some(Connection::Terminal(
        Reader {
            watch: Arc::clone(&watch),
            receive,
            bytes: Vec::new(),
            position: 0,
        },
        Writer {
            watch,
            send: write,
            bytes: Vec::new(),
        },
    )))
}

pub(super) struct Reader {
    watch: Arc<Watch>,
    receive: mpsc::Receiver<io::Result<Vec<u8>>>,
    bytes: Vec<u8>,
    position: usize,
}
impl BufRead for Reader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.position == self.bytes.len() {
            if Instant::now() >= self.watch.budget.question_deadline() {
                return Ok(&[]);
            }
            self.bytes = self
                .watch
                .wait(&self.receive, self.watch.budget.question_deadline())??;
            self.position = 0;
        }
        Ok(&self.bytes[self.position..])
    }
    fn consume(&mut self, amount: usize) {
        self.position = self.bytes.len().min(self.position + amount);
    }
}
impl Read for Reader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let available = self.fill_buf()?;
        let count = available.len().min(bytes.len());
        bytes[..count].copy_from_slice(&available[..count]);
        self.consume(count);
        Ok(count)
    }
}
pub(super) struct Writer {
    watch: Arc<Watch>,
    send: mpsc::SyncSender<(Vec<u8>, mpsc::Sender<io::Result<()>>)>,
    bytes: Vec<u8>,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len() + bytes.len() > 262_144 {
            return Err(io::Error::other("legacy output limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.bytes.is_empty() {
            return Ok(());
        }
        self.watch.check()?;
        let (reply, receive) = mpsc::channel();
        self.send
            .try_send((std::mem::take(&mut self.bytes), reply))
            .map_err(|_| io::Error::other("legacy output busy or ended"))?;
        self.watch.wait(
            &receive,
            (Instant::now() + Duration::from_secs(5)).min(self.watch.budget.overall_deadline()),
        )?
    }
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
    #[test]
    fn scope_and_legacy_confirm_never_imply_terminal_answers() {
        let root = tempfile::tempdir().unwrap();
        let mut args = vec![
            "hive".into(),
            "update".into(),
            "--user-root".into(),
            root.path().to_str().unwrap().into(),
        ];
        assert!(interactive(&args, root.path()).unwrap());
        args.push("--confirm".into());
        assert!(!interactive(&args, root.path()).unwrap());
        args.push("--confirm".into());
        assert!(interactive(&args, root.path()).is_err());
    }
}
