use super::LegacyBudget;
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

const REGISTRY: &str = include_str!("../../../../harness/setup/legacy-update-images.json");
const LIMIT: u64 = 1024 * 1024;

struct Session {
    child: Child,
    send: Sender<Vec<u8>>,
    receive: Receiver<Result<Value, String>>,
    budget: LegacyBudget,
}

type Shared = Rc<RefCell<Session>>;

pub(super) enum Connection {
    Terminal(Reader, Writer),
    Preserve(ParentWatch),
}

pub(super) struct ParentWatch {
    _session: Session,
}

pub(super) struct Reader {
    session: Shared,
    bytes: Vec<u8>,
    position: usize,
    ended: bool,
}

pub(super) struct Writer {
    session: Shared,
    bytes: Vec<u8>,
}

#[derive(Deserialize)]
struct Parent {
    version: String,
    executable: PathBuf,
    arguments: Vec<String>,
}

pub(super) fn connect(root: &Path, budget: LegacyBudget) -> Result<Option<Connection>, String> {
    let mut session = Session::spawn(budget)?;
    let reply = session.exchange(
        &json!({"operation":"inspect","caller_pid":std::process::id()}),
        Instant::now() + Duration::from_secs(20),
    )?;
    match reply["event"].as_str() {
        Some("not-legacy-parent") => return Ok(None),
        Some("verified-parent") => {}
        _ => return Err("legacy updater parent could not be verified; settings preserved".into()),
    }
    let parent: Parent =
        serde_json::from_value(reply).map_err(|_| "invalid legacy parent proof")?;
    let executable = std::env::current_exe().map_err(|_| "current executable is unavailable")?;
    let interactive = interactive_parent(&parent, root, &executable)?;
    let opened = session.exchange(
        &json!({"operation":if interactive {"open"} else {"watch"}}),
        Instant::now() + Duration::from_secs(10),
    )?;
    if !interactive {
        if opened["event"] != "watching" {
            return Err("legacy updater lifetime could not be bound; settings preserved".into());
        }
        return Ok(Some(Connection::Preserve(ParentWatch {
            _session: session,
        })));
    }
    if opened["event"] != "opened" {
        return Err("legacy updater console is unavailable; settings preserved".into());
    }
    let shared = Rc::new(RefCell::new(session));
    Ok(Some(Connection::Terminal(
        Reader {
            session: Rc::clone(&shared),
            bytes: Vec::new(),
            position: 0,
            ended: false,
        },
        Writer {
            session: shared,
            bytes: Vec::new(),
        },
    )))
}

fn interactive_parent(parent: &Parent, root: &Path, executable: &Path) -> Result<bool, String> {
    if !matches!(
        parent.version.as_str(),
        "0.13.1" | "0.13.2-test.1" | "0.13.2-test.2"
    ) || parent.arguments.get(1).map(String::as_str) != Some("update")
    {
        return Err("unsupported legacy updater proof".into());
    }
    let mut options = std::collections::BTreeSet::new();
    let mut specified_root = None;
    let mut direct_target = None;
    let mut confirm = false;
    let mut index = 2;
    while index < parent.arguments.len() {
        let option = parent.arguments[index].as_str();
        if !options.insert(option) {
            return Err("duplicate legacy updater option".into());
        }
        if option == "--confirm" {
            confirm = true;
            index += 1;
            continue;
        }
        let value = parent
            .arguments
            .get(index + 1)
            .ok_or("missing legacy updater option value")?;
        match option {
            "--user-root" => specified_root = Some(Path::new(value)),
            "--direct-handoff" => direct_target = Some(Path::new(value)),
            "--channel" if matches!(value.as_str(), "stable" | "test") => {}
            _ => return Err("unknown legacy updater option".into()),
        }
        index += 2;
    }
    let fallback;
    let original_root = if let Some(root) = specified_root {
        root
    } else {
        fallback = crate::user_install::resolve_user_root_path()?;
        fallback.as_path()
    };
    if !same_path(original_root, root)? {
        return Err("legacy updater user scope differs".into());
    }
    let expected_parent = if let Some(target) = direct_target {
        if !same_path(target, executable)? {
            return Err("legacy direct target differs".into());
        }
        root.join(".hive/runtime/update/hive-direct-update-helper.exe")
    } else {
        executable.to_owned()
    };
    if !same_path(&parent.executable, &expected_parent)? {
        return Err("legacy updater installation scope differs".into());
    }
    // Public predecessor code requires a terminal before this plain-update path can run.
    // Its --confirm mode does not prove whether the original caller wanted input.
    Ok(!confirm)
}

fn same_path(left: &Path, right: &Path) -> Result<bool, String> {
    let left = left
        .canonicalize()
        .map_err(|_| "legacy scope cannot be resolved")?;
    let right = right
        .canonicalize()
        .map_err(|_| "current scope cannot be resolved")?;
    Ok(left
        .to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy()))
}

impl Session {
    fn spawn(budget: LegacyBudget) -> Result<Self, String> {
        use std::os::windows::process::CommandExt;
        let encode = |text: &str| base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
        let registry = serde_json::from_str::<Value>(REGISTRY)
            .map_err(|_| "invalid public parent registry")?
            .to_string();
        let registry_digest = hive_core::sha256_digest(registry.as_bytes());
        let script = include_str!("windows.ps1")
            .replace("{{REGISTRY_SOURCE}}", &registry.replace('\'', "''"))
            .replace("{{REGISTRY_DIGEST}}", &registry_digest)
            .replace(
                "{{PARENT_SOURCE}}",
                &encode(include_str!("windows-parent.cs")),
            )
            .replace(
                "{{CONSOLE_SOURCE}}",
                &encode(include_str!("windows-console.cs")),
            );
        let encoded = encode(&script);
        if encoded.len() > 262_144 {
            return Err("legacy helper source exceeds its bound".into());
        }
        let chunks: Vec<_> = encoded.as_bytes().chunks(16_000).collect();
        let system = std::env::var_os("SystemRoot").ok_or("Windows system root unavailable")?;
        let mut command =
            Command::new(Path::new(&system).join("System32/WindowsPowerShell/v1.0/powershell.exe"));
        // CodeDOM's compiler must not inherit the large, owned script transport environment.
        let bootstrap = format!("$s=[Text.StringBuilder]::new();for($i=0;$i -lt {};$i++){{[void]$s.Append([Environment]::GetEnvironmentVariable('HIVE_UPDATE_BRIDGE_'+$i));[Environment]::SetEnvironmentVariable('HIVE_UPDATE_BRIDGE_'+$i,$null)}};& ([scriptblock]::Create([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($s.ToString()))))", chunks.len());
        for (index, chunk) in chunks.iter().enumerate() {
            command.env(
                format!("HIVE_UPDATE_BRIDGE_{index}"),
                String::from_utf8_lossy(chunk).as_ref(),
            );
        }
        command
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &bootstrap,
            ])
            .creation_flags(0x0800_0000)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command
            .spawn()
            .map_err(|_| "cannot start legacy console helper")?;
        let mut input = child.stdin.take().ok_or("helper input unavailable")?;
        let output = child.stdout.take().ok_or("helper output unavailable")?;
        let (send, requests) = mpsc::channel::<Vec<u8>>();
        let (events, receive) = mpsc::channel();
        let failures = events.clone();
        let writing = budget.clone();
        thread::spawn(move || {
            for bytes in requests {
                if input
                    .write_all(&bytes)
                    .and_then(|()| input.flush())
                    .is_err()
                {
                    writing.disconnect();
                    let _ = failures.send(Err("legacy helper input ended".into()));
                    break;
                }
            }
        });
        let reading = budget.clone();
        thread::spawn(move || {
            let mut output = BufReader::new(output);
            loop {
                let mut bytes = Vec::new();
                let read = (&mut output).take(LIMIT + 1).read_until(b'\n', &mut bytes);
                let event: Result<Value, String> = match read {
                    Ok(count)
                        if count > 0
                            && count <= usize::try_from(LIMIT).unwrap_or(0)
                            && bytes.last() == Some(&b'\n') =>
                    {
                        serde_json::from_slice(&bytes)
                            .map_err(|_| "invalid legacy helper JSON".into())
                    }
                    _ => Err("legacy helper output ended".into()),
                };
                let ended = event.is_err();
                if ended || event.as_ref().is_ok_and(|value| value["event"] == "closed") {
                    reading.disconnect();
                }
                if events.send(event).is_err() || ended {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            send,
            receive,
            budget,
        })
    }

    fn exchange(&mut self, request: &Value, deadline: Instant) -> Result<Value, String> {
        let deadline = deadline.min(self.budget.overall_deadline());
        if Instant::now() >= deadline {
            return Err("legacy update deadline reached".into());
        }
        let mut bytes =
            serde_json::to_vec(request).map_err(|_| "cannot encode legacy helper request")?;
        bytes.push(b'\n');
        if bytes.len() > usize::try_from(LIMIT).unwrap_or(0) {
            return Err("legacy request exceeds its bound".into());
        }
        self.send
            .send(bytes)
            .map_err(|_| "legacy helper input closed")?;
        self.receive
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|_| "legacy helper deadline or disconnect")?
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.budget.disconnect();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Read for Reader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let available = self.fill_buf()?;
        let count = bytes.len().min(available.len());
        bytes[..count].copy_from_slice(&available[..count]);
        self.consume(count);
        Ok(count)
    }
}

impl BufRead for Reader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.position == self.bytes.len() && !self.ended {
            let mut session = self.session.borrow_mut();
            let deadline = session.budget.question_deadline();
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                self.ended = true;
                return Ok(&[]);
            }
            let reply = session.exchange(&json!({"operation":"read","remaining_milliseconds":remaining.as_millis().min(420_000)}), deadline + Duration::from_secs(1))
                .map_err(io::Error::other)?;
            if reply["event"] != "input" {
                return Err(io::Error::other("legacy console disconnected"));
            }
            self.bytes = reply["line"]
                .as_str()
                .map_or_else(Vec::new, |line| line.as_bytes().to_vec());
            self.position = 0;
            self.ended = self.bytes.is_empty();
        }
        Ok(&self.bytes[self.position..])
    }

    fn consume(&mut self, amount: usize) {
        self.position = (self.position + amount).min(self.bytes.len());
    }
}

impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len() + bytes.len() > 262_144 {
            return Err(io::Error::other("legacy console output limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.bytes.is_empty() {
            return Ok(());
        }
        let text = std::str::from_utf8(&self.bytes).map_err(io::Error::other)?;
        let response = self
            .session
            .borrow_mut()
            .exchange(
                &json!({"operation":"write","text":text}),
                Instant::now() + Duration::from_secs(5),
            )
            .map_err(io::Error::other)?;
        if response["event"] != "written" {
            return Err(io::Error::other("legacy console output ended"));
        }
        self.bytes.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_plain_updates_with_the_same_scopes_can_open_input() {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("hive.exe");
        std::fs::write(&executable, b"fixture").unwrap();
        let mut parent = Parent {
            version: "0.13.1".into(),
            executable: executable.clone(),
            arguments: vec![
                "hive.exe".into(),
                "update".into(),
                "--user-root".into(),
                root.path().to_string_lossy().into_owned(),
            ],
        };
        assert!(interactive_parent(&parent, root.path(), &executable).unwrap());
        parent.arguments.push("--confirm".into());
        assert!(!interactive_parent(&parent, root.path(), &executable).unwrap());
        parent.arguments.push("--confirm".into());
        assert!(interactive_parent(&parent, root.path(), &executable).is_err());
        parent.arguments.truncate(4);
        let other = tempfile::tempdir().unwrap();
        assert!(interactive_parent(&parent, other.path(), &executable).is_err());
        parent.version = "0.12.0".into();
        assert!(interactive_parent(&parent, root.path(), &executable).is_err());
    }

    #[test]
    fn public_direct_handoff_uses_its_exact_helper_and_target() {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("bin/hive.exe");
        let helper = root
            .path()
            .join(".hive/runtime/update/hive-direct-update-helper.exe");
        for path in [&executable, &helper] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"fixture").unwrap();
        }
        let mut parent = Parent {
            version: "0.13.1".into(),
            executable: helper.clone(),
            arguments: vec![
                helper.to_string_lossy().into_owned(),
                "update".into(),
                "--direct-handoff".into(),
                executable.to_string_lossy().into_owned(),
                "--user-root".into(),
                root.path().to_string_lossy().into_owned(),
            ],
        };
        assert!(interactive_parent(&parent, root.path(), &executable).unwrap());
        parent.arguments[3] = helper.to_string_lossy().into_owned();
        assert!(interactive_parent(&parent, root.path(), &executable).is_err());
        parent.arguments[3] = executable.to_string_lossy().into_owned();
        parent.executable = root
            .path()
            .join(".hive/runtime/update/hive-update-helper.exe");
        std::fs::write(&parent.executable, b"fixture").unwrap();
        assert!(interactive_parent(&parent, root.path(), &executable).is_err());
    }
}
