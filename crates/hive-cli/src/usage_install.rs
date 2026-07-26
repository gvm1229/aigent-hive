use super::{emit_action_result, ActionResult, Evidence};
use crate::usage::{CommandRunner, SensorError, SystemCommandRunner, UsageHost};
use hive_core::sha256_digest;
use serde_json::json;
use std::process::ExitCode;
use std::time::Duration;

const INSTALL_TIMEOUT: Duration = Duration::from_mins(5);
const OUTPUT_LIMIT: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InstallMode {
    DryRun,
    Apply,
}

#[derive(Debug)]
struct InstallArguments {
    host: UsageHost,
    mode: InstallMode,
    confirmed: bool,
}

struct InstallCommand {
    manager: &'static str,
    arguments: &'static [&'static str],
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    let result = parse(arguments)
        .and_then(|parsed| install(&parsed, &SystemCommandRunner))
        .unwrap_or_else(failure);
    emit_action_result(&result)
}

fn parse(arguments: &[String]) -> Result<InstallArguments, String> {
    if arguments.first().map(String::as_str) != Some("fallback-install") {
        return Err("usage fallback installation requires the fallback-install action".to_owned());
    }
    let mut host = None;
    let mut mode = None;
    let mut output = None;
    let mut confirmed = false;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dry-run" if mode.is_none() => {
                mode = Some(InstallMode::DryRun);
                index += 1;
            }
            "--apply" if mode.is_none() => {
                mode = Some(InstallMode::Apply);
                index += 1;
            }
            "--confirm-install" if !confirmed => {
                confirmed = true;
                index += 1;
            }
            "--dry-run" | "--apply" => {
                return Err("exactly one of --dry-run or --apply is required".to_owned());
            }
            "--confirm-install" => {
                return Err("duplicate option: --confirm-install".to_owned());
            }
            option @ ("--host" | "--output") => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| format!("missing value for {option}"))?;
                let slot = if option == "--host" {
                    &mut host
                } else {
                    &mut output
                };
                if slot.replace(value.clone()).is_some() {
                    return Err(format!("duplicate option: {option}"));
                }
                index += 2;
            }
            option => return Err(format!("unknown option: {option}")),
        }
    }
    let host = match host.as_deref() {
        Some("codex") => UsageHost::Codex,
        Some("claude") => UsageHost::Claude,
        Some("antigravity") => UsageHost::Antigravity,
        Some(_) => return Err("--host must be codex, claude, or antigravity".to_owned()),
        None => return Err("missing required option --host".to_owned()),
    };
    if output.as_deref() != Some("json") {
        return Err("usage fallback installation requires --output json".to_owned());
    }
    let mode = mode.ok_or_else(|| "exactly one of --dry-run or --apply is required".to_owned())?;
    if mode == InstallMode::DryRun && confirmed {
        return Err("--confirm-install is valid only with --apply".to_owned());
    }
    if mode == InstallMode::Apply && !confirmed {
        return Err(
            "CodexBar installation requires explicit current-action consent via --confirm-install"
                .to_owned(),
        );
    }
    Ok(InstallArguments {
        host,
        mode,
        confirmed,
    })
}

fn install(
    arguments: &InstallArguments,
    runner: &impl CommandRunner,
) -> Result<ActionResult, String> {
    let command = supported_command()?;
    let executable = runner
        .qualify(command.manager)
        .map_err(|error| match error {
            SensorError::Unavailable => {
                format!(
                    "supported package manager {} is unavailable",
                    command.manager
                )
            }
            _ => format!(
                "cannot qualify package manager {}: {error}",
                command.manager
            ),
        })?;
    let preview = format_command(command.manager, command.arguments);
    let command_digest = sha256_digest(preview.as_bytes());
    if arguments.mode == InstallMode::DryRun {
        return Ok(ActionResult {
            schema_version: 1,
            action: "InstallUsageFallback",
            status: "success",
            exit_code: 0,
            code: "hive.usage-fallback-install-preview",
            message: format!(
                "CodexBar fallback installation preview prepared for {}",
                arguments.host.as_str()
            ),
            changed_paths: Vec::new(),
            evidence: vec![Evidence {
                kind: "command",
                locator: preview,
                digest: command_digest,
            }],
            next_action: Some(format!(
                "rerun with --host {} --apply --confirm-install to grant one-action installation consent",
                arguments.host.as_str()
            )),
            data: Some(json!({
                "provider": arguments.host.as_str(),
                "fallback": "codexbar",
                "package_manager": command.manager,
                "consent_scope": "current-action",
                "credentials_requested": false
            })),
        });
    }
    debug_assert!(arguments.confirmed);
    let output = runner
        .run(
            &executable,
            command.arguments,
            INSTALL_TIMEOUT,
            OUTPUT_LIMIT,
        )
        .map_err(|error| format!("CodexBar installation command failed: {error}"))?;
    if !output.success {
        return Err("CodexBar installation command exited unsuccessfully".to_owned());
    }
    runner
        .qualify("codexbar")
        .map_err(|_| "CodexBar executable was not available after installation".to_owned())?;
    Ok(ActionResult {
        schema_version: 1,
        action: "InstallUsageFallback",
        status: "success",
        exit_code: 0,
        code: "hive.usage-fallback-installed",
        message: "CodexBar fallback installed after explicit one-action consent".to_owned(),
        changed_paths: Vec::new(),
        evidence: vec![Evidence {
            kind: "command",
            locator: preview,
            digest: command_digest,
        }],
        next_action: None,
        data: Some(json!({
            "provider": arguments.host.as_str(),
            "fallback": "codexbar",
            "package_manager": command.manager,
            "consent_scope": "current-action",
            "credentials_requested": false
        })),
    })
}

#[cfg(target_os = "macos")]
#[allow(clippy::unnecessary_wraps)]
fn supported_command() -> Result<InstallCommand, String> {
    Ok(InstallCommand {
        manager: "brew",
        arguments: &["install", "--cask", "codexbar"],
    })
}

#[cfg(all(target_os = "linux", not(target_arch = "arm")))]
#[allow(clippy::unnecessary_wraps)]
fn supported_command() -> Result<InstallCommand, String> {
    Ok(InstallCommand {
        manager: "brew",
        arguments: &["install", "steipete/tap/codexbar"],
    })
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
#[allow(clippy::unnecessary_wraps)]
fn supported_command() -> Result<InstallCommand, String> {
    Ok(InstallCommand {
        manager: "brew",
        arguments: &["install", "steipete/tap/codexbar"],
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn supported_command() -> Result<InstallCommand, String> {
    Err("CodexBar fallback installation is unsupported on this platform".to_owned())
}

fn format_command(program: &str, arguments: &[&str]) -> String {
    std::iter::once(program)
        .chain(arguments.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}

fn failure(message: String) -> ActionResult {
    let unsupported = message.contains("unsupported on this platform")
        || message.contains("package manager")
        || message.contains("cannot qualify");
    ActionResult {
        schema_version: 1,
        action: "InstallUsageFallback",
        status: if unsupported { "unsupported" } else { "error" },
        exit_code: if unsupported { 4 } else { 2 },
        code: if unsupported {
            "hive.usage-fallback-install-unsupported"
        } else {
            "hive.invalid-input"
        },
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::{CommandOutput, QualifiedExecutable};
    use std::sync::Mutex;

    struct FakeRunner {
        calls: Mutex<Vec<String>>,
    }

    impl FakeRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn qualify(&self, program: &str) -> Result<QualifiedExecutable, SensorError> {
            self.calls
                .lock()
                .expect("calls lock")
                .push(format!("qualify:{program}"));
            Ok(QualifiedExecutable::synthetic(program))
        }

        fn run(
            &self,
            _program: &QualifiedExecutable,
            arguments: &[&str],
            _timeout: Duration,
            _output_limit: usize,
        ) -> Result<CommandOutput, SensorError> {
            self.calls
                .lock()
                .expect("calls lock")
                .push(format!("run:{}", arguments.join(" ")));
            Ok(CommandOutput {
                success: true,
                stdout: Vec::new(),
            })
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn dry_run_never_executes_package_manager() {
        let runner = FakeRunner::new();
        let result = install(
            &InstallArguments {
                host: UsageHost::Claude,
                mode: InstallMode::DryRun,
                confirmed: false,
            },
            &runner,
        )
        .expect("preview");
        assert_eq!(result.code, "hive.usage-fallback-install-preview");
        assert_eq!(
            runner.calls.lock().expect("calls").as_slice(),
            &["qualify:brew"]
        );
    }

    #[test]
    fn apply_requires_confirmation_during_argument_parsing() {
        let error = parse(&[
            "fallback-install".to_owned(),
            "--host".to_owned(),
            "codex".to_owned(),
            "--apply".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect_err("confirmation is required");
        assert!(error.contains("--confirm-install"));
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn confirmed_apply_executes_fixed_command_and_requalifies_codexbar() {
        let runner = FakeRunner::new();
        let result = install(
            &InstallArguments {
                host: UsageHost::Antigravity,
                mode: InstallMode::Apply,
                confirmed: true,
            },
            &runner,
        )
        .expect("install");
        assert_eq!(result.code, "hive.usage-fallback-installed");
        let calls = runner.calls.lock().expect("calls");
        assert_eq!(calls.first().map(String::as_str), Some("qualify:brew"));
        assert!(calls.iter().any(|call| call.starts_with("run:install ")));
        assert_eq!(calls.last().map(String::as_str), Some("qualify:codexbar"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    #[test]
    fn unsupported_platform_never_qualifies_a_package_manager() {
        let runner = FakeRunner::new();
        let Err(error) = install(
            &InstallArguments {
                host: UsageHost::Claude,
                mode: InstallMode::DryRun,
                confirmed: false,
            },
            &runner,
        ) else {
            panic!("unsupported platform must fail");
        };
        assert_eq!(
            error,
            "CodexBar fallback installation is unsupported on this platform"
        );
        assert!(runner.calls.lock().expect("calls").is_empty());
    }
}
