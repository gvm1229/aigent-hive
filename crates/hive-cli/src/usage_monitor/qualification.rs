//! Debug-only, exact-subject acceptance of the production controller. Never installed.
use super::{configuration, controller_scoped, error, load, platform, response, write, CONFIG};
use crate::{
    emit_action_result,
    run::{AdapterError, PinnedTarget},
};
use serde_json::Value;
use std::{path::Path, process::ExitCode};

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
    if arguments.len() != 2 || !platform::valid_thread_id(&arguments[1]) {
        return Err(error("exact fixture root and one test thread are required"));
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
    let mut config = configuration(&fixture)?;
    config.executable = executable;
    write(&root, CONFIG, &config)?;
    controller_scoped(&root, Some(arguments[1].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
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
