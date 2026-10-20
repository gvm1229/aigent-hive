//! Bounded, owned observation workers. They never attach to or signal desktop processes.
use crate::usage::MonitorSensor;
use hive_core::usage_guard::UsageSnapshot;
use process_wrap::std::{ChildWrapper, CommandWrap};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStdin, Command, ExitCode, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

const READ_TIMEOUT: Duration = Duration::from_secs(5);
const INTERNAL: &str = "__usage-observation-worker";

pub(super) fn uptime_ms() -> Option<u64> {
    UPTIME.get().map(|(tick, started)| {
        tick.saturating_add(u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX))
    })
}

static UPTIME: std::sync::OnceLock<(u64, Instant)> = std::sync::OnceLock::new();

pub(super) fn seed_uptime(tick: u64) {
    let _ = UPTIME.set((tick, Instant::now()));
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Request {
    id: u64,
    fresh: bool,
    #[serde(default)]
    catalog: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Response {
    id: u64,
    snapshots: Option<Vec<UsageSnapshot>>,
    contexts: Option<Vec<crate::usage::MonitorThread>>,
    error: Option<String>,
}

#[derive(Debug)]
pub(super) struct Sample {
    pub snapshots: Result<Vec<UsageSnapshot>, String>,
    pub source: &'static str,
    pub primary_error: Option<String>,
}

struct Worker {
    child: Box<dyn ChildWrapper>,
    stdin: ChildStdin,
    replies: Receiver<Result<Response, String>>,
    next_id: u64,
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Job/process-group ownership was established when this helper was spawned.
        // Never enumerate PIDs or signal the user's Codex application.
        let _ = self.child.start_kill();
        let deadline = Instant::now() + Duration::from_millis(100);
        while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Worker {
    fn start() -> Result<Self, String> {
        let executable = std::env::current_exe().map_err(|_| "worker-executable-unavailable")?;
        let mut command = Command::new(executable);
        command
            .arg(INTERNAL)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let mut command = CommandWrap::from(command);
        #[cfg(windows)]
        command.wrap(process_wrap::std::JobObject);
        #[cfg(unix)]
        command.wrap(process_wrap::std::ProcessGroup::leader());
        let mut child = command.spawn().map_err(|_| "worker-spawn-failed")?;
        let stdin = child.stdin().take().ok_or("worker-stdin-unavailable")?;
        let stdout = child.stdout().take().ok_or("worker-stdout-unavailable")?;
        let (send, replies) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = Vec::new();
                let read = reader.by_ref().take(1_048_577).read_until(b'\n', &mut line);
                let result = if !matches!(read, Ok(1..)) {
                    Err("observation-worker-exited".to_owned())
                } else if line.len() > 1_048_576 || line.last() != Some(&b'\n') {
                    Err("observation-worker-output-invalid".to_owned())
                } else {
                    crate::usage::parse_strict_native_json(&line)
                        .map_err(|error| error.to_string())
                        .and_then(|value| {
                            serde_json::from_value(value)
                                .map_err(|_| "observation-worker-protocol-invalid".to_owned())
                        })
                };
                let failed = result.is_err();
                if send.send(result).is_err() || failed {
                    return;
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            replies,
            next_id: 1,
        })
    }

    fn read(&mut self, fresh: bool) -> Result<Vec<UsageSnapshot>, String> {
        let (id, response) = self.exchange(fresh, false)?;
        validate_response(response, id)
    }

    fn catalog(&mut self) -> Result<Vec<crate::usage::MonitorThread>, String> {
        let (_, response) = self.exchange(false, true)?;
        match (response.snapshots, response.contexts, response.error) {
            (None, Some(contexts), None) => Ok(contexts),
            (None, None, Some(error)) => Err(error),
            _ => Err("catalog-response-ambiguous".into()),
        }
    }

    fn exchange(&mut self, fresh: bool, catalog: bool) -> Result<(u64, Response), String> {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or("observation-id-exhausted")?;
        serde_json::to_writer(&mut self.stdin, &Request { id, fresh, catalog })
            .map_err(|_| "observation-write-failed")?;
        self.stdin
            .write_all(b"\n")
            .map_err(|_| "observation-write-failed")?;
        self.stdin.flush().map_err(|_| "observation-write-failed")?;
        let response = self
            .replies
            .recv_timeout(READ_TIMEOUT)
            .map_err(|_| "observation-timeout")??;
        if response.id != id {
            return Err("observation-response-id-mismatch".into());
        }
        Ok((id, response))
    }
}

fn validate_response(response: Response, id: u64) -> Result<Vec<UsageSnapshot>, String> {
    if response.id != id {
        return Err("observation-response-id-mismatch".to_owned());
    }
    match (response.snapshots, response.contexts, response.error) {
        (Some(snapshots), None, None) => Ok(snapshots),
        (None, None, Some(error)) => Err(error),
        _ => Err("observation-response-ambiguous".to_owned()),
    }
}

pub(super) struct Reader {
    primary: Option<Worker>,
}

pub(super) struct CatalogReader {
    worker: Option<Worker>,
}

impl CatalogReader {
    pub fn new() -> Self {
        Self { worker: None }
    }
    pub fn read(&mut self) -> Result<Vec<crate::usage::MonitorThread>, String> {
        if self.worker.is_none() {
            self.worker = Some(Worker::start()?);
        }
        let result = self
            .worker
            .as_mut()
            .expect("catalog worker was created")
            .catalog();
        if result.is_ok() {
            return result;
        }
        self.worker = None;
        let mut fresh = Worker::start()?;
        let result = fresh.catalog();
        if result.is_ok() {
            self.worker = Some(fresh);
        }
        result
    }
}

impl Reader {
    pub fn new() -> Self {
        Self { primary: None }
    }

    #[cfg(debug_assertions)]
    fn primary_pid(&self) -> Option<u32> {
        self.primary.as_ref().map(|worker| worker.child.id())
    }

    pub fn read(&mut self, fresh: bool) -> Sample {
        let primary = if fresh {
            self.primary = None;
            Err("independent-observation-requested".to_owned())
        } else {
            if self.primary.is_none() {
                match Worker::start() {
                    Ok(worker) => self.primary = Some(worker),
                    Err(error) => return self.recover(error),
                }
            }
            self.primary
                .as_mut()
                .expect("worker was created")
                .read(false)
        };
        match primary {
            Ok(snapshots) => Sample {
                snapshots: Ok(snapshots),
                source: "persistent",
                primary_error: None,
            },
            Err(error) => self.recover(error),
        }
    }

    fn recover(&mut self, primary_error: String) -> Sample {
        self.primary = None;
        let snapshots = Worker::start().and_then(|mut worker| worker.read(true));
        Sample {
            snapshots,
            source: "fresh-process",
            primary_error: Some(primary_error),
        }
    }
}

/// Read-only source qualification: the production reader and validator, without desktop control.
/// No settings, snapshots, account identifiers or conversation content are written or returned.
#[cfg(debug_assertions)]
pub(crate) fn run_supervisor() -> ExitCode {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct QualificationRequest {
        id: u64,
        #[serde(default)]
        fresh: bool,
        #[serde(default)]
        exit_owned_reader: bool,
    }
    let mut reader = Reader::new();
    let mut state = super::state::MonitorState::new();
    let mut cadence = super::state::Cadence::new();
    let started = Instant::now();
    let mut independent = false;
    let mut previous_id = 0;
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    loop {
        let mut line = Vec::new();
        if !matches!(
            input.by_ref().take(1_025).read_until(b'\n', &mut line),
            Ok(1..)
        ) {
            return ExitCode::SUCCESS;
        }
        let request = crate::usage::parse_strict_native_json(&line)
            .ok()
            .and_then(|value| serde_json::from_value::<QualificationRequest>(value).ok());
        let Some(request) = request.filter(|request| {
            line.len() <= 1_024 && line.last() == Some(&b'\n') && request.id > previous_id
        }) else {
            return ExitCode::from(2);
        };
        previous_id = request.id;
        if request.exit_owned_reader {
            if let Some(worker) = reader.primary.as_mut() {
                // Fault injection holds only the query helper created by this reader.
                let _ = worker.child.start_kill();
            }
        }
        cadence.begin(started.elapsed());
        let sample = reader.read(request.fresh || independent);
        let gap_before = cadence.gap(started.elapsed());
        let (valid, needs_independent) = state.record_query(sample, super::now());
        independent = needs_independent;
        cadence.complete(started.elapsed(), valid);
        let response = serde_json::json!({
            "id":request.id, "valid":valid, "source":state.last_query_source,
            "primary_error":state.primary_query_error, "error":state.last_query_error,
            "primary_worker_pid":reader.primary_pid(), "observations":state.observation_count,
            "consecutive_failures":state.consecutive_query_failures,
            "gap_before_ms":gap_before.as_millis(), "blind_before":gap_before.as_secs() >= super::BLIND_SECONDS,
            "elapsed_ms":started.elapsed().as_millis(),
            "next_in_ms":cadence.next_sample.saturating_sub(started.elapsed()).as_millis(),
        });
        if serde_json::to_writer(&mut output, &response).is_err()
            || output.write_all(b"\n").is_err()
            || output.flush().is_err()
        {
            return ExitCode::from(2);
        }
    }
}

/// Internal JSONL protocol, deliberately outside the public monitor command grammar.
pub(crate) fn run_worker() -> ExitCode {
    let mut sensor = MonitorSensor::new();
    let input = std::io::stdin();
    let mut input = input.lock();
    let mut output = std::io::stdout().lock();
    loop {
        let mut line = Vec::new();
        if !matches!(
            input.by_ref().take(1_025).read_until(b'\n', &mut line),
            Ok(1..)
        ) {
            return ExitCode::SUCCESS;
        }
        if line.len() > 1_024 || line.last() != Some(&b'\n') {
            return ExitCode::from(2);
        }
        let request = crate::usage::parse_strict_native_json(&line)
            .ok()
            .and_then(|value| serde_json::from_value::<Request>(value).ok());
        let Some(request) = request else {
            return ExitCode::from(2);
        };
        if request.catalog && request.fresh {
            return ExitCode::from(2);
        }
        let response = if request.catalog {
            match sensor.desktop_threads() {
                Ok(contexts) => Response {
                    id: request.id,
                    contexts: Some(contexts),
                    snapshots: None,
                    error: None,
                },
                Err(error) => Response {
                    id: request.id,
                    contexts: None,
                    snapshots: None,
                    error: Some(error.to_string()),
                },
            }
        } else {
            let result = if request.fresh {
                MonitorSensor::fresh()
            } else {
                sensor.read()
            };
            match result {
                Ok(snapshot) => Response {
                    id: request.id,
                    snapshots: Some(snapshot.core_snapshots()),
                    contexts: None,
                    error: None,
                },
                Err(error) => Response {
                    id: request.id,
                    snapshots: None,
                    contexts: None,
                    error: Some(error.to_string()),
                },
            }
        };
        if serde_json::to_writer(&mut output, &response).is_err()
            || output.write_all(b"\n").is_err()
            || output.flush().is_err()
        {
            return ExitCode::from(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn late_or_ambiguous_replies_never_supply_an_observation() {
        assert!(validate_response(
            Response {
                id: 2,
                snapshots: Some(vec![]),
                contexts: Some(vec![]),
                error: None,
            },
            2
        )
        .is_err());
        assert!(validate_response(
            Response {
                id: 1,
                snapshots: Some(vec![]),
                contexts: None,
                error: None
            },
            2
        )
        .is_err());
        assert!(validate_response(
            Response {
                id: 2,
                snapshots: Some(vec![]),
                contexts: None,
                error: Some("failure".into())
            },
            2
        )
        .is_err());
        assert!(validate_response(
            Response {
                id: 2,
                snapshots: None,
                contexts: None,
                error: None
            },
            2
        )
        .is_err());
        assert_eq!(
            validate_response(
                Response {
                    id: 2,
                    snapshots: Some(vec![]),
                    contexts: None,
                    error: None
                },
                2
            ),
            Ok(vec![])
        );
    }
}
