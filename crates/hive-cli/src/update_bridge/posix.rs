//! Shared POSIX terminal I/O; each platform authenticates its own parent first.
#[cfg(target_os = "linux")]
use super::linux::Parent;
#[cfg(target_os = "macos")]
use super::macos::Parent;
use super::LegacyBudget;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

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

pub(super) fn interactive(arguments: &[String], root: &Path) -> Result<bool, String> {
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

pub(super) fn connect(
    parent: Parent,
    budget: LegacyBudget,
    needs_terminal: bool,
) -> Result<Connection, String> {
    if !parent.alive() {
        return Err("legacy parent changed".into());
    }
    let watch = Arc::new(Watch { parent, budget });
    let weak = Arc::downgrade(&watch);
    thread::spawn(move || {
        while let Some(watch) = weak.upgrade() {
            if watch.check().is_err() {
                break;
            }
            drop(watch);
            thread::sleep(Duration::from_millis(100));
        }
    });
    if !needs_terminal {
        return Ok(Connection::Preserve(watch));
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
    Ok(Connection::Terminal(
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
    ))
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
