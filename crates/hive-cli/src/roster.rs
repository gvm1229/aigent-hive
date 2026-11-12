//! Read-only roster and host-reported capability inspection.
use crate::run::{parse_options, read_explicit_file, required};
use hive_core::roster::{CapabilityReport, Roster};
use serde_json::{json, Value};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "hive roster [check --role <name> --capabilities <host-report.json>] --target <dir> --user-root <home> --output json\n";

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.iter().any(|s| s == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match execute(arguments) {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                json!({"status":"unsupported","code":"hive.roster-unsupported","message":error,"enabled":false})
            );
            ExitCode::from(2)
        }
    }
}

fn execute(arguments: &[String]) -> Result<Value, String> {
    let check = arguments.first().is_some_and(|s| s == "check");
    let options = parse_options(
        if check { &arguments[1..] } else { arguments },
        &["--target", "--user-root", "--role", "--capabilities"],
    )
    .map_err(|e| e.message().to_owned())?;
    let target = required(&options, "--target").map_err(|e| e.message().to_owned())?;
    let user = required(&options, "--user-root").map_err(|e| e.message().to_owned())?;
    let roster = Roster::load(Path::new(user), Path::new(target))?;
    if !check {
        return Ok(json!({"roster":roster,"enabled":false}));
    }
    let role = required(&options, "--role").map_err(|e| e.message().to_owned())?;
    let report_path = required(&options, "--capabilities").map_err(|e| e.message().to_owned())?;
    let bytes = read_explicit_file(Path::new(report_path), 1024 * 1024)
        .map_err(|e| e.message().to_owned())?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    hive_core::validate_json_schema(
        include_str!("../../../schemas/roster-capability-report.schema.json"),
        &value,
        "roster capability report",
    )?;
    let report: CapabilityReport = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    roster.check(role, &report)?;
    Ok(
        json!({"status":"host-reported-supported","enabled":false,"role":role,"provider":report.provider,"version":report.version,"evidence_limit":"supplied official output only; no authenticated runtime round trip"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roster_cli_defaults_and_unknown_options() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().to_str().unwrap();
        let args = ["--target", path, "--user-root", path, "--output", "json"].map(str::to_owned);
        assert_eq!(execute(&args).unwrap()["enabled"], false);
        assert!(execute(&["--unknown".into()]).is_err());
    }
}
