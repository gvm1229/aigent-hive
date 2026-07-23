use hive_core::{ensure_consumer_target, source_marker_path};
use std::env;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "\
Aigent Hive source scaffold

USAGE:
    hive doctor
    hive check-target <path>
";

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    match arguments.next().as_deref() {
        Some("doctor") => {
            reject_extra_arguments(arguments)?;
            doctor()
        }
        Some("check-target") => {
            let target = arguments
                .next()
                .ok_or_else(|| format!("missing target path\n\n{USAGE}"))?;
            reject_extra_arguments(arguments)?;
            check_target(Path::new(&target))
        }
        Some("-h" | "--help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(command) => Err(format!("unknown command: {command}\n\n{USAGE}")),
    }
}

fn reject_extra_arguments(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    if let Some(argument) = arguments.next() {
        return Err(format!("unexpected argument: {argument}\n\n{USAGE}"));
    }
    Ok(())
}

fn doctor() -> Result<(), String> {
    let current_directory =
        env::current_dir().map_err(|error| format!("cannot read current directory: {error}"))?;
    let marker = source_marker_path(&current_directory);

    println!("aigent-hive {}", env!("CARGO_PKG_VERSION"));
    println!("workspace: {}", current_directory.display());
    println!(
        "source workspace: {}",
        if marker.is_file() { "yes" } else { "no" }
    );
    println!("model API client: disabled by architecture");
    println!("setup renderer: not implemented in Phase 0");
    Ok(())
}

fn check_target(target: &Path) -> Result<(), String> {
    ensure_consumer_target(target).map_err(|error| error.to_string())?;
    println!("target accepted: {}", target.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn help_is_the_default_command() {
        assert_eq!(run(std::iter::empty()), Ok(()));
    }

    #[test]
    fn unknown_commands_fail() {
        let error = run(["unknown".to_owned()].into_iter()).expect_err("command should fail");
        assert!(error.contains("unknown command"));
    }
}
