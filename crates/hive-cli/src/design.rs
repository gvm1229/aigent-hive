//! Read-only view of the bundled design sources owned by Aphrodite.
use serde_json::{json, Value};
use std::process::ExitCode;

const USAGE: &str = "hive design [status] [--output json]\n";
const UPSTREAM_LOCK: &str = include_str!("../../../harness/upstream/UPSTREAM.lock");
const CONFLICTS: &str =
    include_str!("../../../harness/skills/design-curate/references/conflicts.md");

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.iter().any(|argument| argument == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let value = match execute(arguments) {
        Ok(value) => value,
        Err(error) => {
            let value = json!({"schema_version":1,"status":"error","code":"hive.design-invalid-input","message":error});
            if !crate::human_output::emit_if_human(&value) {
                println!("{value}");
            }
            return ExitCode::from(2);
        }
    };
    if !crate::human_output::emit_if_human(&value) {
        println!("{value}");
    }
    ExitCode::SUCCESS
}

fn execute(arguments: &[String]) -> Result<Value, String> {
    let mut rest = arguments.iter().map(String::as_str).peekable();
    if rest.peek() == Some(&"status") {
        rest.next();
    }
    while let Some(argument) = rest.next() {
        match argument {
            "--output" if rest.next() == Some("json") => {}
            other => return Err(format!("unsupported design argument: {other}")),
        }
    }
    status()
}

fn status() -> Result<Value, String> {
    let lock: Value = serde_json::from_str(UPSTREAM_LOCK)
        .map_err(|error| format!("bundled source lock is invalid: {error}"))?;
    let sources = lock["sources"]
        .as_array()
        .ok_or("bundled source lock has no sources")?
        .iter()
        .filter(|source| {
            source["used_by"]
                .as_array()
                .is_some_and(|skills| skills.iter().any(|skill| skill == "design-curate"))
        })
        .map(|source| {
            json!({
                "name": source["name"],
                "repository": source["repo"],
                "commit": source["commit"],
                "license": source["license"],
                "synced_at": source["synced_at"],
            })
        })
        .collect::<Vec<_>>();
    let rulings = CONFLICTS
        .lines()
        .filter(|line| {
            line.starts_with("| ") && !line.starts_with("| Topic") && !line.starts_with("| ---")
        })
        .count();
    Ok(json!({
        "schema_version": 1,
        "status": "ok",
        "skill": "design-curate",
        "sources": sources,
        "conflict_rulings": rulings,
        "update": "Bundled sources change only with a Hive update.",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_status_lists_the_four_bundled_sources_and_rulings() {
        let value = execute(&["status".to_owned()]).expect("status");
        let names = value["sources"]
            .as_array()
            .expect("sources")
            .iter()
            .map(|source| source["name"].as_str().expect("name"))
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "hallmark",
                "frontend-design",
                "web-artifacts-builder",
                "impeccable"
            ]
        );
        assert!(value["conflict_rulings"].as_u64().expect("count") >= 5);
        assert!(execute(&[]).is_ok());
        assert!(execute(&["sync".to_owned()]).is_err());
    }
}
