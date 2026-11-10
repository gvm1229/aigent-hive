//! Read-only storage report and a preview-first cleanup of retired monitor executables.

use crate::{emit_action_result, ActionResult};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const PREFIX: &str = "[Hades]";
const HIVE: &str = ".hive";
const MONITOR_BIN: &str = ".hive/runtime/usage-monitor/bin";
const MONITOR_CONFIG: &str = ".hive/runtime/usage-monitor/config.json";
const MONITOR_JOURNAL: &str = ".hive/runtime/usage-monitor/upgrade.json";

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
            Err(error) => return Err(format!("cannot list {}: {error}", directory.display())),
        };
        for entry in entries {
            let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
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

/// Executables referenced by the saved monitor configuration or its last update record.
fn referenced_monitor_binaries(user_root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let read = |relative: &str| -> Option<Value> {
        serde_json::from_slice(&fs::read(user_root.join(relative)).ok()?).ok()
    };
    let mut executables = Vec::new();
    if let Some(config) = read(MONITOR_CONFIG) {
        executables.push(config["executable"].clone());
    }
    if let Some(journal) = read(MONITOR_JOURNAL) {
        executables.push(journal["to"]["executable"].clone());
        executables.push(journal["from"]["executable"].clone());
    }
    for executable in executables {
        if let Some(name) = executable.as_str().and_then(|path| {
            Path::new(&path.replace('\\', "/"))
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        }) {
            names.push(name);
        }
    }
    names
}

/// Monitor executables that neither the configuration nor the last update record uses.
fn prune_candidates(user_root: &Path) -> Result<Vec<(PathBuf, u64)>, String> {
    let keep = referenced_monitor_binaries(user_root);
    let directory = user_root.join(MONITOR_BIN);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("cannot list {}: {error}", directory.display())),
    };
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| format!("cannot inspect {name}: {error}"))?;
        let is_exe = Path::new(&name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"));
        if metadata.is_file() && is_exe && name.starts_with("hive-") && !keep.contains(&name) {
            candidates.push((entry.path(), metadata.len()));
        }
    }
    candidates.sort();
    Ok(candidates)
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
    emit_action_result(&ActionResult {
        schema_version: 1,
        action,
        status: "failed",
        exit_code: 2,
        code: "hive.storage-input-invalid",
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

pub(crate) fn run_storage(arguments: &[String]) -> ExitCode {
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
    let options = match parse(arguments, true) {
        Ok(options) => options,
        Err(message) => return failure("PruneStorage", message),
    };
    let candidates = match prune_candidates(&options.user_root) {
        Ok(candidates) => candidates,
        Err(message) => return failure("PruneStorage", message),
    };
    let confirm = options.mode == Some("confirm");
    let (mut removed, mut skipped) = (Vec::new(), Vec::new());
    if confirm {
        for (path, _) in &candidates {
            match fs::remove_file(path) {
                Ok(()) => removed.push(path.display().to_string()),
                Err(error) => skipped.push(json!({"path":path,"reason":error.to_string()})),
            }
        }
    }
    let listed: Vec<Value> = candidates
        .iter()
        .map(|(path, bytes)| json!({"path":path,"bytes":bytes,"category":"monitor-binaries"}))
        .collect();
    let reclaim: u64 = candidates.iter().map(|(_, bytes)| bytes).sum();
    if !options.json {
        for (path, bytes) in &candidates {
            let verb = if confirm { "removed" } else { "would remove" };
            println!(
                "{PREFIX} {verb}: {} ({})",
                path.display(),
                human_size(*bytes)
            );
        }
        println!(
            "{PREFIX} {} candidate(s), {}{}",
            candidates.len(),
            human_size(reclaim),
            if confirm {
                ""
            } else {
                "; run with --confirm to delete"
            }
        );
        return ExitCode::SUCCESS;
    }
    emit_action_result(&result(
        "PruneStorage",
        if confirm {
            "hive.storage-pruned"
        } else {
            "hive.storage-prune-previewed"
        },
        format!(
            "{PREFIX} {} unused monitor executable(s){}",
            candidates.len(),
            if confirm {
                " processed"
            } else {
                " listed; nothing changed"
            }
        ),
        removed.clone(),
        json!({"mode":options.mode,"candidates":listed,"reclaimable_bytes":reclaim,
            "removed":removed,"skipped":skipped}),
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

    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), ".hive/knowledge/Wiki/a.md", &[1; 100]);
        write(root.path(), ".hive/index/hive.sqlite3", &[2; 200]);
        write(root.path(), ".hive/index/vector/v.bin", &[3; 300]);
        write(root.path(), ".hive/backups/user-install/b.tar", &[4; 400]);
        write(root.path(), ".hive/config/user-setup.yml", &[5; 50]);
        for name in [
            "hive-a.exe",
            "hive-b.exe",
            "hive-c.exe",
            "hive-old.exe",
            "hive-new.exe",
        ] {
            write(root.path(), &format!("{MONITOR_BIN}/{name}"), &[6; 10]);
        }
        let bin = root.path().join(MONITOR_BIN);
        write(
            root.path(),
            MONITOR_CONFIG,
            serde_json::to_string(&json!({"executable":bin.join("hive-new.exe")}))
                .unwrap()
                .as_bytes(),
        );
        write(
            root.path(),
            MONITOR_JOURNAL,
            serde_json::to_string(&json!({"from":{"executable":bin.join("hive-old.exe")},
                "to":{"executable":bin.join("hive-new.exe")}}))
            .unwrap()
            .as_bytes(),
        );
        root
    }

    #[test]
    fn storage_categories_sum_to_every_regular_file_byte() {
        let root = fixture();
        let (categories, total) = measure(root.path()).unwrap();
        let by_name = |name: &str| categories.iter().find(|row| row.0 == name).unwrap().1;
        assert_eq!(by_name("knowledge"), 100);
        assert_eq!(by_name("index"), 200);
        assert_eq!(by_name("vector"), 300);
        assert_eq!(by_name("backups"), 400);
        assert_eq!(by_name("monitor-binaries"), 50);
        assert_eq!(categories.iter().map(|row| row.1).sum::<u64>(), total);
        let expected: u64 = walk_sizes(&root.path().join(HIVE));
        assert_eq!(total, expected);
    }

    fn walk_sizes(path: &Path) -> u64 {
        fs::read_dir(path)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk_sizes(&path)
                } else {
                    fs::metadata(path).unwrap().len()
                }
            })
            .sum()
    }

    #[test]
    fn dry_run_lists_unused_executables_and_changes_nothing() {
        let root = fixture();
        let before = snapshot(root.path());
        let candidates = prune_candidates(root.path()).unwrap();
        let names: Vec<String> = candidates
            .iter()
            .map(|(path, _)| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["hive-a.exe", "hive-b.exe", "hive-c.exe"]);
        assert_eq!(
            run_prune(&[
                "--dry-run".into(),
                "--user-root".into(),
                root.path().display().to_string(),
                "--output".into(),
                "json".into(),
            ]),
            ExitCode::SUCCESS
        );
        assert_eq!(snapshot(root.path()), before);
    }

    #[test]
    fn prune_requires_an_explicit_mode() {
        assert!(parse(&[], true).is_err());
        assert!(parse(&["--dry-run".into(), "--confirm".into()], true).is_err());
    }

    fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    files.push((path.clone(), fs::read(path).unwrap()));
                }
            }
        }
        files.sort();
        files
    }
}
