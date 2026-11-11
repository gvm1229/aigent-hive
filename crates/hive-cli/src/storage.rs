//! Read-only storage report and a preview-first cleanup of retired monitor executables.

use crate::{emit_action_result, ActionResult};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const PREFIX: &str = "[Hades]";
const HIVE: &str = ".hive";
const MONITOR_BIN: &str = ".hive/runtime/usage-monitor/bin";

/// Ordered categories; the first matching prefix owns a file.
const CATEGORIES: &[(&str, &str)] = &[
    ("vector", ".hive/index/vector"),
    ("index", ".hive/index"),
    ("knowledge", ".hive/knowledge"),
    ("monitor-binaries", MONITOR_BIN),
    ("backups", ".hive/backups"),
];

struct Options {
    user_root: PathBuf,
    json: bool,
    mode: Option<&'static str>,
}

fn parse(arguments: &[String], prune: bool) -> Result<Options, String> {
    let mut user_root = None;
    let mut json = false;
    let mut mode = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--user-root" => {
                index += 1;
                let value = arguments.get(index).ok_or("--user-root requires a path")?;
                user_root = Some(PathBuf::from(value));
            }
            "--output" => {
                index += 1;
                if arguments.get(index).map(String::as_str) != Some("json") {
                    return Err("--output accepts only json".to_owned());
                }
                json = true;
            }
            "--dry-run" if prune && mode.is_none() => mode = Some("dry-run"),
            "--confirm" if prune && mode.is_none() => mode = Some("confirm"),
            other => return Err(format!("unsupported option: {other}")),
        }
        index += 1;
    }
    if prune && mode.is_none() {
        return Err("prune requires --dry-run or --confirm".to_owned());
    }
    let user_root = match user_root {
        Some(root) => root,
        None => crate::user_install::resolve_user_root_path()?,
    };
    if !user_root.is_absolute() {
        return Err("--user-root must be absolute".to_owned());
    }
    Ok(Options {
        user_root,
        json,
        mode,
    })
}

/// Sum regular-file bytes per category without following symbolic links.
/// Category name, regular-file bytes and file count.
type CategoryRow = (String, u64, u64);

fn measure(user_root: &Path) -> Result<(Vec<CategoryRow>, u64), String> {
    let mut totals: Vec<CategoryRow> = CATEGORIES
        .iter()
        .map(|(name, _)| ((*name).to_owned(), 0, 0))
        .chain(std::iter::once(("other".to_owned(), 0, 0)))
        .collect();
    let mut total = 0;
    let root = user_root.join(HIVE);
    let mut pending = vec![root.clone()];
    while let Some(directory) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "read: cannot list {}: {error}",
                    directory.display()
                ))
            }
        };
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("read: cannot read directory entry: {error}"))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("read: cannot inspect {}: {error}", path.display()))?;
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                let relative = path
                    .strip_prefix(user_root)
                    .map_err(|_| "storage walk escaped the user root".to_owned())?
                    .to_string_lossy()
                    .replace('\\', "/");
                let slot = CATEGORIES
                    .iter()
                    .position(|(_, prefix)| {
                        relative == *prefix || relative.starts_with(&format!("{prefix}/"))
                    })
                    .unwrap_or(CATEGORIES.len());
                totals[slot].1 += metadata.len();
                totals[slot].2 += 1;
                total += metadata.len();
            }
        }
    }
    Ok((totals, total))
}

fn result(
    action: &'static str,
    code: &'static str,
    message: String,
    changed: Vec<String>,
    data: Value,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message,
        changed_paths: changed,
        evidence: Vec::new(),
        next_action: None,
        data: Some(data),
    }
}

fn failure(action: &'static str, message: String) -> ExitCode {
    // Read failures are verification errors; everything else is invalid input.
    let (status, exit_code, code, message) = match message.strip_prefix("read: ") {
        Some(read) => (
            "verification-failed",
            1,
            "hive.storage-read-failed",
            read.to_owned(),
        ),
        None => ("failed", 2, "hive.storage-input-invalid", message),
    };
    emit_action_result(&ActionResult {
        schema_version: 1,
        action,
        status,
        exit_code,
        code,
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    })
}

fn human_size(bytes: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let value = bytes as f64;
    if bytes >= 1 << 30 {
        format!("{:.1} GiB", value / f64::from(1_u32 << 30))
    } else if bytes >= 1 << 20 {
        format!("{:.1} MiB", value / f64::from(1_u32 << 20))
    } else if bytes >= 1 << 10 {
        format!("{:.1} KiB", value / f64::from(1_u32 << 10))
    } else {
        format!("{bytes} B")
    }
}

pub(crate) const STORAGE_USAGE: &str = "Usage:\n  hive storage [--user-root <path>] [--output json]\n  hive prune --dry-run|--confirm [--user-root <path>] [--output json]\n\nstorage reports Hive file bytes by category and changes nothing.\nprune lists or removes monitor executables that the saved monitor records do not use.\n";

fn is_help(arguments: &[String]) -> bool {
    arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
}

pub(crate) fn run_storage(arguments: &[String]) -> ExitCode {
    if is_help(arguments) {
        print!("{STORAGE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let options = match parse(arguments, false) {
        Ok(options) => options,
        Err(message) => return failure("ReportStorage", message),
    };
    let (categories, total) = match measure(&options.user_root) {
        Ok(measured) => measured,
        Err(message) => return failure("ReportStorage", message),
    };
    if !options.json {
        for (name, bytes, files) in &categories {
            println!("{PREFIX} {name}: {} ({files} files)", human_size(*bytes));
        }
        println!("{PREFIX} total: {}", human_size(total));
        return ExitCode::SUCCESS;
    }
    let rows: Vec<Value> = categories
        .iter()
        .map(|(name, bytes, files)| json!({"category":name,"bytes":bytes,"files":files}))
        .collect();
    emit_action_result(&result(
        "ReportStorage",
        "hive.storage-reported",
        format!("{PREFIX} Hive storage measured without changes"),
        Vec::new(),
        json!({"user_root":options.user_root,"categories":rows,"total_bytes":total}),
    ))
}

pub(crate) fn run_prune(arguments: &[String]) -> ExitCode {
    if is_help(arguments) {
        print!("{STORAGE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let options = match parse(arguments, true) {
        Ok(options) => options,
        Err(message) => return failure("PruneStorage", message),
    };
    let confirm = options.mode == Some("confirm");
    let report = match crate::usage_monitor::prune_unused(&options.user_root, confirm) {
        Ok(report) => report,
        Err(error) => {
            return emit_action_result(&ActionResult {
                schema_version: 1,
                action: "PruneStorage",
                status: error.status(),
                exit_code: error.exit_code(),
                code: "hive.storage-prune-refused",
                message: format!("{PREFIX} {}", error.message()),
                changed_paths: Vec::new(),
                evidence: Vec::new(),
                next_action: Some("inspect hive usage monitor status; nothing was removed".into()),
                data: None,
            });
        }
    };
    let list = |key: &str| report[key].as_array().cloned().unwrap_or_default();
    let (candidates, removed, skipped) = (list("candidates"), list("removed"), list("skipped"));
    if !options.json {
        for item in &candidates {
            let bytes = item["bytes"].as_u64().unwrap_or(0);
            println!(
                "{PREFIX} would remove: {} ({})",
                item["path"].as_str().unwrap_or("?"),
                human_size(bytes)
            );
        }
        for item in &removed {
            println!("{PREFIX} removed: {}", item.as_str().unwrap_or("?"));
        }
        for item in &skipped {
            println!(
                "{PREFIX} skipped: {} ({})",
                item["path"].as_str().unwrap_or("?"),
                item["reason"].as_str().unwrap_or("unknown")
            );
        }
        if confirm {
            println!(
                "{PREFIX} {} removed, {} skipped",
                removed.len(),
                skipped.len()
            );
        } else {
            println!(
                "{PREFIX} {} candidate(s); run with --confirm to delete",
                candidates.len()
            );
        }
        return ExitCode::SUCCESS;
    }
    let changed = removed
        .iter()
        .filter_map(|path| path.as_str().map(str::to_owned))
        .collect();
    emit_action_result(&result(
        "PruneStorage",
        if confirm {
            "hive.storage-pruned"
        } else {
            "hive.storage-prune-previewed"
        },
        if confirm {
            format!(
                "{PREFIX} {} removed, {} skipped",
                removed.len(),
                skipped.len()
            )
        } else {
            format!(
                "{PREFIX} {} unused monitor executable(s) listed; nothing changed",
                candidates.len()
            )
        },
        changed,
        json!({"mode":options.mode,"report":report}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn storage_categories_sum_to_every_regular_file_byte() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), ".hive/knowledge/Wiki/a.md", &[1; 100]);
        write(root.path(), ".hive/index/hive.sqlite3", &[2; 200]);
        write(root.path(), ".hive/index/vector/v.bin", &[3; 300]);
        write(root.path(), ".hive/backups/user-install/b.tar", &[4; 400]);
        write(root.path(), ".hive/config/user-setup.yml", &[5; 50]);
        write(
            root.path(),
            ".hive/runtime/usage-monitor/bin/hive-a.exe",
            &[6; 10],
        );
        let (categories, total) = measure(root.path()).unwrap();
        let by_name = |name: &str| categories.iter().find(|row| row.0 == name).unwrap().1;
        assert_eq!(by_name("knowledge"), 100);
        assert_eq!(by_name("index"), 200);
        assert_eq!(by_name("vector"), 300);
        assert_eq!(by_name("backups"), 400);
        assert_eq!(by_name("monitor-binaries"), 10);
        assert_eq!(by_name("other"), 50);
        assert_eq!(categories.iter().map(|row| row.1).sum::<u64>(), total);
        assert_eq!(total, 1060);
    }

    #[test]
    fn prune_requires_an_explicit_mode() {
        assert!(parse(&[], true).is_err());
        assert!(parse(&["--dry-run".into(), "--confirm".into()], true).is_err());
        assert!(is_help(&["--help".into()]));
    }
}
