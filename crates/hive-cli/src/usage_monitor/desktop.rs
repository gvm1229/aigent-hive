//! Existing desktop coordination pipe; never starts a model or signals an app process.
use super::platform;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::Stdio;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

const SCRIPT: &str = include_str!("desktop.ps1");

pub(super) struct Desktop {
    commands: SyncSender<Value>,
    pub events: Receiver<Value>,
}

impl Desktop {
    pub fn start_scoped(qualification: Option<String>) -> Self {
        let (commands, receive) = mpsc::sync_channel(4);
        let (events, stream) = mpsc::sync_channel(16);
        thread::spawn(move || run(&receive, &events, qualification.as_deref()));
        Self {
            commands,
            events: stream,
        }
    }

    pub fn stop(&self, digest: &str, detected_at_ms: Option<u64>) {
        self.stop_scoped(digest, detected_at_ms, None);
    }

    pub fn stop_scoped(&self, digest: &str, detected_at_ms: Option<u64>, scope: Option<&[String]>) {
        let _ = self.commands.try_send(
            json!({"operation":"stop","incident":digest,"detected_at_ms":detected_at_ms,"scope":scope}),
        );
    }

    pub fn clear(&self) {
        let _ = self.commands.try_send(json!({"operation":"clear"}));
    }
}

fn catalog_worker(qualification: Option<String>) -> (Receiver<Value>, SyncSender<()>) {
    let (send, receive) = mpsc::sync_channel(1);
    let (refresh, requests) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut reader = super::sensor::CatalogReader::new();
        loop {
            let started = Instant::now();
            let value = match reader.read() {
                Ok(threads) => {
                    json!({"operation":"discover","valid":true,"ids":threads.iter().filter(|thread| qualification.as_deref().is_none_or(|id| id == thread.id)).map(|thread| &thread.id).collect::<Vec<_>>(),"contexts":threads})
                }
                Err(_) => json!({"operation":"discover","valid":false,"ids":[]}),
            };
            if matches!(
                send.try_send(value),
                Err(mpsc::TrySendError::Disconnected(_))
            ) {
                return;
            }
            if let Err(mpsc::RecvTimeoutError::Disconnected) =
                requests.recv_timeout(Duration::from_secs(15).saturating_sub(started.elapsed()))
            {
                return;
            }
        }
    });
    (receive, refresh)
}

fn write_command(writer: &mut impl Write, value: &Value) -> std::io::Result<()> {
    serde_json::to_writer(&mut *writer, value).map_err(std::io::Error::other)?;
    writer.write_all(b"\n")
}

fn unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(u128::MAX, |at| at.as_millis())
}

fn release_control(pending: &Value, delivered: Option<&str>, reconnect: bool, at: u128) -> bool {
    pending["operation"] == "stop"
        && (reconnect && pending["incident"].as_str() == delivered
            || pending["detected_at_ms"].as_u64().is_none_or(|detected| {
                at < u128::from(detected) || at >= u128::from(detected) + 5500
            }))
}

fn read_packets(stdout: impl Read, send_lines: &SyncSender<Value>) {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut line = Vec::new();
        let result = reader.by_ref().take(262_145).read_until(b'\n', &mut line);
        if !matches!(result, Ok(1..)) || line.len() > 262_144 {
            return;
        }
        let value = crate::usage::parse_strict_native_json(&line);
        let Ok(value) = value else { return };
        if send_lines.send(value).is_err() {
            return;
        }
    }
}

fn run(commands: &Receiver<Value>, events: &SyncSender<Value>, qualification: Option<&str>) {
    let (catalogs, refresh) = catalog_worker(qualification.map(str::to_owned));
    let mut catalog = json!({"operation":"discover","valid":false,"ids":[]});
    let mut catalog_at = Instant::now();
    let mut pending = json!({"operation":"clear"});
    let mut delivered_incident: Option<String> = None;
    loop {
        if let Ok(command) = commands.try_recv() {
            pending = command;
        }
        if release_control(&pending, delivered_incident.as_deref(), true, unix_ms()) {
            pending = json!({"operation":"clear"});
        }
        let Ok(mut builder) = platform::command_scoped(SCRIPT, qualification) else {
            return;
        };
        let child = builder
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = child else {
            if events.send(json!({"connected":false})).is_err() {
                return;
            }
            thread::sleep(Duration::from_secs(1));
            continue;
        };
        let Some(mut stdin) = child.stdin.take() else {
            return;
        };
        let Some(stdout) = child.stdout.take() else {
            return;
        };
        let (send_lines, lines) = mpsc::sync_channel(8);
        thread::spawn(move || read_packets(stdout, &send_lines));
        let started = Instant::now();
        let mut last_packet = started;
        let mut sent = Value::Null;
        let mut sent_catalog = Value::Null;
        loop {
            while let Ok(command) = commands.try_recv() {
                pending = command;
            }
            if release_control(&pending, delivered_incident.as_deref(), false, unix_ms()) {
                pending = json!({"operation":"clear"});
            }
            if pending != sent {
                if pending["operation"] == "stop" {
                    // Interrupt cached owners now; refresh metadata without delaying cancellation.
                    let _ = refresh.try_send(());
                }
                // Only one small bounded command is outstanding per incident transition.
                if write_command(&mut stdin, &pending).is_err() {
                    break;
                }
                if pending["operation"] == "stop" {
                    delivered_incident = pending["incident"].as_str().map(str::to_owned);
                }
                sent = pending.clone();
            }
            let mut refreshed = false;
            while let Ok(value) = catalogs.try_recv() {
                let _ = events.try_send(
                    json!({"type":"catalog","valid":value["valid"],"contexts":value["contexts"]}),
                );
                catalog = json!({"operation":"discover","valid":value["valid"],"ids":value["ids"]});
                catalog_at = Instant::now();
                refreshed = true;
            }
            if catalog_at.elapsed() >= Duration::from_secs(20) {
                catalog = json!({"operation":"discover","valid":false,"ids":[]});
            }
            if refreshed || catalog != sent_catalog {
                if write_command(&mut stdin, &catalog).is_err() {
                    break;
                }
                sent_catalog = catalog.clone();
            }
            while let Ok(packet) = lines.try_recv() {
                last_packet = Instant::now();
                if events.try_send(packet).is_err() {
                    // A stopped controller must not accumulate unbounded native data.
                    break;
                }
            }
            if !matches!(child.try_wait(), Ok(None))
                || last_packet.elapsed() >= Duration::from_secs(8)
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        // These are this monitor's helper processes, never Codex application processes.
        let _ = child.kill();
        let _ = child.wait();
        if events.send(json!({"connected":false})).is_err() {
            return;
        }
        thread::sleep(Duration::from_secs(1));
    }
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn one_shot_delivery_is_not_replayed_after_reconnect_or_its_deadline() {
        let event =
            serde_json::json!({"operation":"stop","incident":"event","detected_at_ms":10000});
        assert!(!super::release_control(&event, None, false, 10010));
        assert!(!super::release_control(&event, Some("event"), false, 10010));
        assert!(super::release_control(&event, Some("event"), true, 10010));
        assert!(!super::release_control(&event, Some("older"), true, 10010));
        assert!(super::release_control(&event, None, false, 15500));
        assert!(super::release_control(&event, None, false, 9999));
        assert!(super::release_control(
            &serde_json::json!({"operation":"stop"}),
            None,
            false,
            10010
        ));
        assert!(!super::release_control(
            &serde_json::json!({"operation":"clear"}),
            None,
            true,
            10010
        ));
    }

    #[test]
    fn production_adapter_fits_the_windows_launch_budget() {
        assert!(super::platform::command(super::SCRIPT).is_ok());
    }
}
