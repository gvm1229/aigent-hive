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
    pub fn start() -> Self {
        let (commands, receive) = mpsc::sync_channel(4);
        let (events, stream) = mpsc::sync_channel(16);
        thread::spawn(move || run(&receive, &events));
        Self {
            commands,
            events: stream,
        }
    }

    pub fn stop(&self, digest: &str, detected_at_ms: Option<u64>) {
        let _ = self.commands.try_send(
            json!({"operation":"stop","incident":digest,"detected_at_ms":detected_at_ms}),
        );
    }

    pub fn clear(&self) {
        let _ = self.commands.try_send(json!({"operation":"clear"}));
    }
}

fn catalog_worker() -> (Receiver<Value>, SyncSender<()>) {
    let (send, receive) = mpsc::sync_channel(1);
    let (refresh, requests) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut reader = crate::usage::MonitorSensor::new();
        loop {
            let started = Instant::now();
            let value = match reader.desktop_thread_ids() {
                Ok(ids) => json!({"operation":"discover","valid":true,"ids":ids}),
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

fn run(commands: &Receiver<Value>, events: &SyncSender<Value>) {
    let (catalogs, refresh) = catalog_worker();
    let mut catalog = json!({"operation":"discover","valid":false,"ids":[]});
    let mut catalog_at = Instant::now();
    let mut pending = json!({"operation":"clear"});
    loop {
        if let Ok(command) = commands.try_recv() {
            pending = command;
        }
        let Ok(mut builder) = platform::command(SCRIPT) else {
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
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = Vec::new();
                let result = reader.by_ref().take(16385).read_until(b'\n', &mut line);
                if !matches!(result, Ok(1..)) || line.len() > 16384 {
                    return;
                }
                let value = crate::usage::parse_strict_native_json(&line);
                let Ok(value) = value else { return };
                if send_lines.send(value).is_err() {
                    return;
                }
            }
        });
        let started = Instant::now();
        let mut last_packet = started;
        let mut sent = Value::Null;
        let mut sent_catalog = Value::Null;
        loop {
            while let Ok(command) = commands.try_recv() {
                pending = command;
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
                sent = pending.clone();
            }
            let mut refreshed = false;
            while let Ok(value) = catalogs.try_recv() {
                catalog = value;
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
