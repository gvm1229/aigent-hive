//! Debug-only, exact-subject acceptance of the production controller. Never installed.
use super::{configuration, controller_scoped, error, load, platform, response, write, CONFIG};
use crate::{
    emit_action_result,
    run::{AdapterError, PinnedTarget},
};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    subject: String,
    mode: String,
    samples: AtomicUsize,
}

static FIXTURE: OnceLock<Fixture> = OnceLock::new();

pub(super) fn scope_contexts(root: &Path, contexts: &mut Vec<crate::usage::MonitorThread>) {
    if let Some(fixture) = FIXTURE.get().filter(|fixture| fixture.root == root) {
        contexts.retain(|context| context.id == fixture.subject);
        for context in contexts {
            // Only the isolated collector uses this source-policy fixture. The real
            // conversation remains projectless and cannot inherit its caller's bypass.
            context.cwd = Some(fixture.source.clone());
            context.parent_id = None;
        }
    }
}

pub(super) fn observation(root: &Path, mut sample: super::sensor::Sample) -> super::sensor::Sample {
    if let Some(fixture) = FIXTURE
        .get()
        .filter(|fixture| fixture.root == root && fixture.mode != "live")
    {
        if let Ok(snapshots) = &mut sample.snapshots {
            let index = fixture.samples.fetch_add(1, Ordering::Relaxed);
            for snapshot in snapshots {
                snapshot.remaining_percent = scenario_remaining(&fixture.mode, index);
            }
            sample.source = "qualification";
        }
    }
    sample
}

fn scenario_remaining(mode: &str, index: usize) -> f64 {
    match (mode, index == 0) {
        ("five", true) => 5.01,
        ("five", false) => 5.0,
        ("zero", true) | ("reset", true) => 1.0,
        ("zero", false) => 0.0,
        ("reset", false) => 2.0,
        _ => unreachable!("qualification mode was validated before activation"),
    }
}

pub(crate) fn run_qualification(arguments: &[String]) -> ExitCode {
    let outcome = qualify(arguments);
    let result = match outcome {
        Ok(data) => response(
            "success",
            0,
            "hive.usage-qualification-complete",
            "scoped qualification completed",
            Some(data),
        ),
        Err(error) => response(
            error.status(),
            error.exit_code(),
            "hive.usage-qualification-blocked",
            error.message(),
            None,
        ),
    };
    emit_action_result(&result)
}

fn qualify(arguments: &[String]) -> Result<Value, AdapterError> {
    if !(2..=3).contains(&arguments.len()) || !platform::valid_thread_id(&arguments[1]) {
        return Err(error("exact fixture root and one test thread are required"));
    }
    let mode = arguments.get(2).map_or("live", String::as_str);
    if !matches!(mode, "live" | "five" | "zero" | "reset") {
        return Err(error("unknown qualification observation scenario"));
    }
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|_| error("source workspace missing"))?;
    if !source.join("hive-source.json").is_file() {
        return Err(error("source-only qualification requires source marker"));
    }
    let fixture = Path::new(&arguments[0])
        .canonicalize()
        .map_err(|_| error("qualification fixture missing"))?;
    if !fixture.starts_with(source.join("tests/work/usage-monitor-acceptance"))
        || fixture == source.join("tests/work/usage-monitor-acceptance")
    {
        return Err(error(
            "qualification root outside owned disposable fixtures",
        ));
    }
    let executable = std::env::current_exe().map_err(|_| error("unknown executable"))?;
    if !executable
        .canonicalize()
        .map_err(|_| error("qualification executable missing"))?
        .starts_with(source.join("target/debug"))
    {
        return Err(error("qualification requires source debug executable"));
    }
    let root = PinnedTarget::open_usage(&fixture)?;
    root.read_required(Path::new(".hive/config/user-setup.yml"), 65536)?;
    let marker: Value = load(&root, ".hive/install/codex.json")?
        .ok_or_else(|| error("qualification fixture marker missing"))?;
    if marker["acceptance_fixture"] != true {
        return Err(error("qualification fixture marker differs"));
    }
    FIXTURE
        .set(Fixture {
            root: fixture.clone(),
            source,
            subject: arguments[1].clone(),
            mode: mode.into(),
            samples: AtomicUsize::new(0),
        })
        .map_err(|_| error("qualification already configured"))?;
    let mut config = configuration(&fixture)?;
    config.executable = executable;
    write(&root, CONFIG, &config)?;
    controller_scoped(&root, Some(arguments[1].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controlled_samples_exercise_reset_and_floor_without_changing_identity() {
        use hive_core::{
            sha256_digest,
            usage_guard::{
                evaluate_usage, SourceConfidence, UsageDecision, UsagePolicy, UsageSnapshot,
                UsageWindow,
            },
        };
        for (mode, threshold) in [("five", 5), ("zero", 0), ("reset", 0)] {
            let account = sha256_digest(b"isolated-account");
            let mut state = super::super::MonitorState::new();
            let mut sample = UsageSnapshot {
                schema_version: 1,
                sensor_id: "codex-app-server".into(),
                sensor_version: "fixture".into(),
                host_scope: "codex".into(),
                account_scope_digest: account.clone(),
                quota_pool: None,
                quota_window: UsageWindow::Weekly,
                remaining_percent: scenario_remaining(mode, 0),
                measured_at_unix_seconds: 100,
                expires_at_unix_seconds: 160,
                resets_at_unix_seconds: 1000,
                source_confidence: SourceConfidence::High,
            };
            state
                .observe_confirmed(&[sample.clone()], 100, false)
                .unwrap();
            sample.remaining_percent = scenario_remaining(mode, 1);
            sample.measured_at_unix_seconds = 115;
            state
                .observe_confirmed(&[sample.clone()], 115, false)
                .unwrap();
            assert_eq!(state.incident.is_some(), mode == "reset");
            let policy = UsagePolicy::new("codex-app-server", "fixture", "codex", account)
                .with_stop_remaining_percent(threshold)
                .unwrap();
            assert_eq!(
                matches!(
                    evaluate_usage(&policy, &[sample], &[], 115),
                    UsageDecision::Block(_)
                ),
                mode != "reset"
            );
        }
    }
    #[test]
    fn qualification_refuses_ordinary_roots_and_shell_text() {
        assert!(qualify(&[]).is_err());
        assert!(qualify(&[
            "C:/Users/example".into(),
            "00000000-0000-0000-0000-000000000000".into()
        ])
        .is_err());
        assert!(!platform::valid_thread_id(
            "00000000-0000-0000-0000-00000000000'"
        ));
        assert!(platform::valid_thread_id(
            "00000000-0000-0000-0000-000000000000"
        ));
    }
}
