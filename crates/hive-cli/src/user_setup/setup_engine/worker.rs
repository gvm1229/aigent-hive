//! A legacy updater's apply/validate phase runs in an owned, time-bounded process tree.
use crate::update_bridge::LegacyBudget;
use crate::user_setup::{self, Arguments, SetupBase, SetupError, SetupMode};
use crate::ActionResult;
use base64::Engine;
use process_wrap::std::{ChildWrapper, CommandWrap};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::sync::{mpsc, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const INPUT_LIMIT: u64 = 4 * 1024 * 1024;
const OUTPUT_LIMIT: u64 = 4 * 1024 * 1024;
static WORKER_DEADLINE: OnceLock<Instant> = OnceLock::new();

pub(in crate::user_setup) fn command_timeout(requested: Duration) -> Duration {
    WORKER_DEADLINE.get().map_or(requested, |deadline| {
        requested.min(deadline.saturating_duration_since(Instant::now()))
    })
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema_version: u8,
    package_version: String,
    catalog_digest: String,
    deadline_unix_millis: u64,
    user_root: PathBuf,
    operation_id: String,
    answers: String,
    expected_config: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    schema_version: u8,
    phase: String,
    exit_code: u8,
    message: String,
    changed_paths: Vec<String>,
}

pub(in crate::user_setup) fn run() -> ExitCode {
    let result = receive_and_apply();
    let reply = match result {
        Ok(result) => Reply {
            schema_version: 1,
            phase: "validated".into(),
            exit_code: 0,
            message: result.message,
            changed_paths: result.changed_paths,
        },
        Err(failure) => {
            let result = user_setup::failure(&failure);
            Reply {
                schema_version: 1,
                phase: "incomplete".into(),
                exit_code: result.exit_code,
                message: result.message,
                changed_paths: Vec::new(),
            }
        }
    };
    println!(
        "{}",
        serde_json::to_string(&reply).expect("typed setup worker reply")
    );
    ExitCode::from(reply.exit_code)
}

fn receive_and_apply() -> Result<ActionResult, SetupError> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .lock()
        .take(INPUT_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| SetupError::Input("setup worker input ended".into()))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > INPUT_LIMIT {
        return Err(SetupError::Input(
            "setup worker input exceeds its bound".into(),
        ));
    }
    let request: Request = serde_json::from_slice(&bytes)
        .map_err(|_| SetupError::Input("invalid setup worker request".into()))?;
    let remaining = request
        .deadline_unix_millis
        .saturating_sub(unix_millis()?)
        .min(90_000);
    WORKER_DEADLINE.get_or_init(|| Instant::now() + Duration::from_millis(remaining));
    apply_request(&request)
}

fn apply_request(request: &Request) -> Result<ActionResult, SetupError> {
    apply_request_using(request, user_setup::execute_answers)
}

fn apply_request_using(
    request: &Request,
    mut execute: impl FnMut(&Arguments, &[u8], SetupBase<'_>) -> Result<ActionResult, SetupError>,
) -> Result<ActionResult, SetupError> {
    if request.schema_version != 1
        || request.package_version != env!("HIVE_PACKAGE_VERSION")
        || request.catalog_digest != hive_core::sha256_digest(super::QUESTIONS.as_bytes())
        || !request.user_root.is_absolute()
    {
        return Err(SetupError::Conflict(
            "setup worker version or scope changed".into(),
        ));
    }
    let remaining = request
        .deadline_unix_millis
        .saturating_sub(unix_millis()?)
        .min(90_000);
    if remaining == 0 {
        return Err(SetupError::Conflict("setup worker deadline expired".into()));
    }
    let decode = |value: &str| -> Result<Vec<u8>, SetupError> {
        let value = base64::engine::general_purpose::STANDARD
            .decode(value)
            .map_err(|_| SetupError::Input("invalid setup worker payload".into()))?;
        if u64::try_from(value.len()).unwrap_or(u64::MAX) > user_setup::MAX_USER_SETUP_BYTES {
            return Err(SetupError::Input(
                "setup worker payload exceeds its bound".into(),
            ));
        }
        Ok(value)
    };
    let answers = decode(&request.answers)?;
    let expected = request.expected_config.as_deref().map(decode).transpose()?;
    let (root, cap) = crate::user_install::open_canonical_user_root_for_setup(&request.user_root)
        .map_err(SetupError::Conflict)?;
    let _lease = crate::user_install::lock_user_setup(&cap).map_err(SetupError::Conflict)?;
    let store = user_setup::progress_store::ProgressStore::open(&request.user_root, None)?;
    let answer_value = serde_json::from_slice(&answers)
        .map_err(|_| SetupError::Input("invalid setup worker answers".into()))?;
    let progress = store.require_applying(&request.operation_id, &answer_value)?;
    let mut arguments = Arguments {
        answers: "stdin:setup-apply-worker".into(),
        mode: SetupMode::Apply,
        user_root: root,
        root_cap: cap,
    };
    let applied = execute(
        &arguments,
        &answers,
        SetupBase::Expected(expected.as_deref()),
    )?;
    arguments.mode = SetupMode::Validate;
    execute(&arguments, &answers, SetupBase::Current)?;
    // Record success before writing the pipe; losing the reply must not repeat installation.
    user_setup::progress_store::ProgressStore::open(&arguments.user_root, None)?
        .record_completion(&progress)?;
    Ok(applied)
}

pub(super) fn apply(
    arguments: &Arguments,
    answers: &[u8],
    expected: Option<&[u8]>,
    operation_id: &str,
    budget: &LegacyBudget,
) -> Result<ActionResult, SetupError> {
    let deadline = budget.apply_deadline().ok_or_else(|| {
        SetupError::Conflict("legacy apply reserve is unavailable; resume this update".into())
    })?;
    if !budget.is_connected() {
        return Err(SetupError::Conflict(
            "updater connection ended before apply".into(),
        ));
    }
    let encode = |value: &[u8]| base64::engine::general_purpose::STANDARD.encode(value);
    let request = Request {
        schema_version: 1,
        package_version: env!("HIVE_PACKAGE_VERSION").into(),
        catalog_digest: hive_core::sha256_digest(super::QUESTIONS.as_bytes()),
        deadline_unix_millis: unix_millis()?.saturating_add(
            u64::try_from(
                deadline
                    .saturating_duration_since(Instant::now())
                    .as_millis(),
            )
            .unwrap_or(0),
        ),
        user_root: arguments.user_root.clone(),
        operation_id: operation_id.to_owned(),
        answers: encode(answers),
        expected_config: expected.map(encode),
    };
    let input =
        serde_json::to_vec(&request).map_err(|error| SetupError::Internal(error.to_string()))?;
    if u64::try_from(input.len()).unwrap_or(u64::MAX) > INPUT_LIMIT {
        return Err(SetupError::Input(
            "setup worker request exceeds its bound".into(),
        ));
    }
    let mut command = Command::new(
        std::env::current_exe()
            .map_err(|_| SetupError::Internal("setup executable unavailable".into()))?,
    );
    command
        .arg("__setup-apply-worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = spawn_owned(command)?;
    let mut stdin = child.stdin().take().expect("piped worker input");
    let stdout = child.stdout().take().expect("piped worker output");
    thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let ok = stdout
            .take(OUTPUT_LIMIT + 1)
            .read_to_end(&mut bytes)
            .is_ok()
            && u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= OUTPUT_LIMIT;
        let _ = send.send(ok.then_some(bytes));
    });
    let status = wait(&mut *child, budget, deadline)?;
    let output = receive
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .ok()
        .flatten()
        .ok_or_else(|| SetupError::Verification("setup worker result is incomplete".into()))?;
    let reply: Reply = serde_json::from_slice(&output)
        .map_err(|_| SetupError::Verification("invalid setup worker result".into()))?;
    if reply.schema_version != 1
        || !status.success()
        || reply.exit_code != 0
        || reply.phase != "validated"
    {
        return Err(SetupError::Verification(reply.message));
    }
    Ok(ActionResult {
        schema_version: 1,
        action: "SetupHiveUser",
        status: "success",
        exit_code: 0,
        code: "hive.user-setup-complete",
        message: reply.message,
        changed_paths: reply.changed_paths,
        evidence: Vec::new(),
        next_action: None,
        data: None,
    })
}

fn spawn_owned(command: Command) -> Result<Box<dyn ChildWrapper>, SetupError> {
    let mut wrapped = CommandWrap::from(command);
    #[cfg(windows)]
    {
        // The wrapper exposes its dependency-owned flag type without a second Win32 dependency.
        #[allow(clippy::default_trait_access)]
        let mut flags = process_wrap::std::CreationFlags(Default::default());
        flags.0 .0 = 0x0800_0000;
        wrapped.wrap(flags).wrap(process_wrap::std::JobObject);
    }
    #[cfg(unix)]
    {
        wrapped.wrap(process_wrap::std::ProcessGroup::leader());
    }
    wrapped
        .spawn()
        .map_err(|_| SetupError::Internal("cannot start owned setup worker".into()))
}

fn wait(
    child: &mut dyn ChildWrapper,
    budget: &LegacyBudget,
    deadline: Instant,
) -> Result<std::process::ExitStatus, SetupError> {
    loop {
        if !budget.is_connected() || Instant::now() >= deadline {
            terminate(child, budget.overall_deadline());
            return Err(SetupError::Verification(
                "setup apply was interrupted; saved progress needs recovery".into(),
            ));
        }
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => {
                terminate(child, budget.overall_deadline());
                return Err(SetupError::Verification(
                    "setup worker state is unavailable; inspect saved progress".into(),
                ));
            }
        }
    }
}

fn terminate(child: &mut dyn ChildWrapper, overall: Instant) {
    let _ = child.start_kill();
    let until = overall.min(Instant::now() + Duration::from_secs(30));
    while Instant::now() < until {
        if !matches!(child.try_wait(), Ok(None)) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn unix_millis() -> Result<u64, SetupError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|value| u64::try_from(value.as_millis()).ok())
        .ok_or_else(|| SetupError::Internal("setup worker clock is unavailable".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(root: &std::path::Path) -> Request {
        Request {
            schema_version: 1,
            package_version: env!("HIVE_PACKAGE_VERSION").into(),
            catalog_digest: hive_core::sha256_digest(super::super::QUESTIONS.as_bytes()),
            deadline_unix_millis: unix_millis().unwrap() + 90_000,
            user_root: root.to_owned(),
            operation_id: "a".repeat(64),
            answers: base64::engine::general_purpose::STANDARD.encode(b"{}"),
            expected_config: None,
        }
    }

    #[test]
    fn worker_persists_completion_before_reply_and_rejects_duplicate_activation() {
        use user_setup::progress_store::{ProgressPhase, ProgressStore};
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let answers = serde_json::json!({"interface_language":"ko"});
        let mut progress = store
            .new_progress("user-language".into(), answers.clone())
            .unwrap();
        progress.phase = ProgressPhase::Applying;
        store.save(&store.load().unwrap(), &progress).unwrap();
        store.record_applying(&progress).unwrap();
        let mut request = request(root.path());
        request.operation_id.clone_from(&progress.operation_id);
        request.answers =
            base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&answers).unwrap());
        let (_, cap) =
            crate::user_install::open_canonical_user_root_for_setup(root.path()).unwrap();
        let lease = crate::user_install::lock_user_setup(&cap).unwrap();
        assert!(apply_request_using(&request, |_, _, _| panic!("concurrent activation")).is_err());
        drop(lease);
        let mut phases = Vec::new();
        let result = apply_request_using(&request, |arguments, _, _| {
            assert!(crate::user_install::lock_user_setup(&cap).is_err());
            phases.push(arguments.mode);
            Ok(ActionResult {
                schema_version: 1,
                action: "SetupHiveUser",
                status: "success",
                exit_code: 0,
                code: "hive.user-setup-valid",
                message: "fixture validation".into(),
                changed_paths: Vec::new(),
                evidence: Vec::new(),
                next_action: None,
                data: None,
            })
        })
        .unwrap();
        assert_eq!(phases, [SetupMode::Apply, SetupMode::Validate]);
        // Discard the reply: a restarted parent still sees completion and cannot apply again.
        drop(result);
        assert_eq!(
            store.load().unwrap().progress.unwrap().phase,
            ProgressPhase::Complete
        );
        store.record_completion(&progress).unwrap();
        assert!(apply_request_using(&request, |_, _, _| panic!("duplicate activation")).is_err());
        let mut forged = progress.clone();
        forged.answers["interface_language"] = serde_json::json!("en");
        assert!(store.record_completion(&forged).is_err());
    }

    #[test]
    fn changed_saved_answers_never_reach_the_worker_application() {
        use user_setup::progress_store::{ProgressPhase, ProgressStore};
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = store
            .new_progress(
                "user-language".into(),
                serde_json::json!({"interface_language":"ko"}),
            )
            .unwrap();
        progress.phase = ProgressPhase::Applying;
        store.save(&store.load().unwrap(), &progress).unwrap();
        store.record_applying(&progress).unwrap();
        let mut request = request(root.path());
        request.operation_id.clone_from(&progress.operation_id);
        assert!(
            apply_request_using(&request, |_, _, _| panic!("changed answers applied")).is_err()
        );
        assert_eq!(store.load().unwrap().progress.unwrap(), progress);
    }

    #[test]
    fn validation_failure_never_records_completion() {
        use user_setup::progress_store::{ProgressPhase, ProgressStore};
        let root = tempfile::tempdir().unwrap();
        let store = ProgressStore::open(root.path(), None).unwrap();
        let mut progress = store
            .new_progress("user-language".into(), serde_json::json!({}))
            .unwrap();
        progress.phase = ProgressPhase::Applying;
        store.save(&store.load().unwrap(), &progress).unwrap();
        store.record_applying(&progress).unwrap();
        let mut request = request(root.path());
        request.operation_id.clone_from(&progress.operation_id);
        let mut phases = Vec::new();
        assert!(apply_request_using(&request, |arguments, _, _| {
            phases.push(arguments.mode);
            if arguments.mode == SetupMode::Validate {
                return Err(SetupError::Verification("fixture validation failed".into()));
            }
            Ok(ActionResult {
                schema_version: 1,
                action: "SetupHiveUser",
                status: "success",
                exit_code: 0,
                code: "hive.user-setup-valid",
                message: "fixture apply".into(),
                changed_paths: Vec::new(),
                evidence: Vec::new(),
                next_action: None,
                data: None,
            })
        })
        .is_err());
        assert_eq!(phases, [SetupMode::Apply, SetupMode::Validate]);
        assert_eq!(store.load().unwrap().progress.unwrap(), progress);
    }

    #[test]
    fn worker_rejects_changed_version_scope_or_config_before_host_mutation() {
        let root = tempfile::tempdir().unwrap();
        let mut changed = request(root.path());
        changed.package_version = "0.13.1".into();
        assert!(apply_request(&changed).is_err());
        assert!(!root.path().join(".hive").exists());
        let mut changed = request(root.path());
        changed.user_root = "relative".into();
        assert!(apply_request(&changed).is_err());
        let path = root.path().join(user_setup::USER_SETUP_RELATIVE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"concurrent edit").unwrap();
        let Err(failure) = apply_request(&request(root.path())) else {
            panic!("changed settings accepted")
        };
        assert!(matches!(failure, SetupError::Conflict(_)));
        assert_eq!(std::fs::read(path).unwrap(), b"concurrent edit");
        assert!(!root.path().join(".hive/install").exists());
    }

    fn sleeping_command() -> Command {
        #[cfg(windows)]
        {
            let mut command = Command::new(
                PathBuf::from(std::env::var_os("SystemRoot").unwrap())
                    .join("System32/WindowsPowerShell/v1.0/powershell.exe"),
            );
            command.args([
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-Command",
                "Start-Sleep -Seconds 30",
            ]);
            command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            command
        }
        #[cfg(not(windows))]
        {
            let mut command = Command::new("/bin/sh");
            command
                .args(["-c", "sleep 30"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            command
        }
    }

    #[test]
    fn deadline_or_disconnected_terminal_stops_the_owned_worker() {
        for disconnect in [false, true] {
            let budget = LegacyBudget::testing();
            let mut child = spawn_owned(sleeping_command()).unwrap();
            if disconnect {
                budget.disconnect();
            }
            let began = Instant::now();
            assert!(wait(&mut *child, &budget, began + Duration::from_millis(100)).is_err());
            assert!(began.elapsed() < Duration::from_secs(5));
            assert!(child.try_wait().unwrap().is_some());
        }
    }
}
