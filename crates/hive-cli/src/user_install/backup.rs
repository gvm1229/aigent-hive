//! Verified copies outside `.hive` (OV160-023).
//!
//! `.hive/backups` sits inside the trees that reinstall and uninstall of 0.15.3 and earlier
//! delete, so recovery copies live in `<user-root>/hive-backups` and Hive never prunes them.
use super::{
    capability_metadata, file_permissions, installed_unix_mode, io_internal, json_line,
    lock_user_setup, open_canonical_user_root, portable, read_optional_regular, unix_seconds,
    validate_relative, write_atomic_with_permissions, FilePermissions, InstallError,
};
use crate::ActionResult;
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

const ROOT: &str = "hive-backups";
const MANIFEST: &str = "manifest.json";
const FILE_LIMIT: u64 = 128 * 1024 * 1024;
const TOTAL_LIMIT: u64 = 512 * 1024 * 1024;

/// Host files that Hive edits. Nothing else in a host's own folders is read: no sign-in data.
const HOST_FILES: &[&str] = &[
    ".claude/settings.json",
    ".claude/CLAUDE.md",
    ".codex/AGENTS.md",
    ".codex/AGENTS.override.md",
    ".gemini/GEMINI.md",
];
const HIVE_FILES: &[&str] = &[
    ".hive/roster.toml",
    ".hive/state/claude-statusline.json",
    ".hive/runtime/usage-monitor/config.json",
    ".hive/runtime/usage-monitor/disable.json",
    ".hive/install/codex.json",
    ".hive/install/claude.json",
    ".hive/install/antigravity.json",
    ".hive/install/user-projection.json",
];
const REGENERABLE: &[&str] = &[
    ".hive/marketplaces",
    ".hive/plugins",
    ".hive/index",
    ".hive/install-transactions",
];
const PROJECTION_ROOTS: &[&str] = &[
    ".hive",
    ".agents",
    ".gemini/config",
    ".claude/skills",
    ".codex/skills",
];
/// A default restore returns only what a person owns. Guides, runtime state, claims, knowledge
/// and install records can disagree with the installed version, so they need `--path`.
const DEFAULT_RESTORE: &[&str] = &[
    ".hive/config/projects.yml",
    ".hive/config/collections.yml",
    ".hive/config/user-setup.yml",
    ".hive/config/user-feature-answers.yml",
    ".hive/config/user-preferences.json",
    ".hive/roster.toml",
];

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    digest: String,
    executable: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u8,
    operation: String,
    created_at: u64,
    product_version: String,
    entries: Vec<Entry>,
    skipped: Vec<String>,
}

/// A path that a backup may read and a restore may write.
fn in_scope(path: &Path) -> bool {
    validate_relative(path).is_ok()
        && path.extension().is_none_or(|extension| extension != "lock")
        && (HOST_FILES.iter().any(|file| path == Path::new(file))
            || PROJECTION_ROOTS.iter().any(|root| path.starts_with(root)))
}

fn private_mode() -> Option<u32> {
    installed_unix_mode(false).map(|_| 0o600)
}

fn write_private(root: &Dir, path: &Path, bytes: &[u8]) -> Result<(), InstallError> {
    write_atomic_with_permissions(
        root,
        path,
        bytes,
        FilePermissions {
            executable: false,
            unix_mode: private_mode(),
        },
        None,
        None,
    )
}

fn standard_paths(root: &Dir) -> Result<BTreeSet<PathBuf>, InstallError> {
    let mut paths: BTreeSet<PathBuf> = HOST_FILES
        .iter()
        .chain(HIVE_FILES)
        .map(PathBuf::from)
        .collect();
    let hive = Path::new(".hive");
    let config = Path::new(".hive/config");
    let opened = match root.open_dir_nofollow(hive) {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(paths),
        Err(error) => return Err(io_internal("open", hive, error)),
    };
    let directory = match opened.open_dir_nofollow("config") {
        Ok(directory) => directory,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(paths),
        Err(error) => return Err(io_internal("open", config, error)),
    };
    for entry in directory
        .entries()
        .map_err(|error| io_internal("list", config, error))?
    {
        let entry = entry.map_err(|error| io_internal("list", config, error))?;
        if entry
            .file_type()
            .map_err(|error| io_internal("inspect", config, error))?
            .is_file()
        {
            paths.insert(config.join(entry.file_name()));
        }
    }
    Ok(paths)
}

/// Every regular file below `tree`, without following links.
pub(super) fn tree_files(root: &Dir, tree: &Path) -> Result<Vec<PathBuf>, InstallError> {
    let mut found = Vec::new();
    let mut pending = vec![tree.to_path_buf()];
    while let Some(relative) = pending.pop() {
        let mut directory = root
            .try_clone()
            .map_err(|error| io_internal("pin", &relative, error))?;
        let mut missing = false;
        for component in relative.components() {
            match directory.open_dir_nofollow(component.as_os_str()) {
                Ok(next) => directory = next,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    missing = true;
                    break;
                }
                Err(error) => return Err(io_internal("open", &relative, error)),
            }
        }
        if missing {
            continue;
        }
        for entry in directory
            .entries()
            .map_err(|error| io_internal("list", &relative, error))?
        {
            let entry = entry.map_err(|error| io_internal("list", &relative, error))?;
            let kind = entry
                .file_type()
                .map_err(|error| io_internal("inspect", &relative, error))?;
            let path = relative.join(entry.file_name());
            if kind.is_dir() {
                pending.push(path);
            } else if kind.is_file() {
                found.push(path);
            }
        }
    }
    Ok(found)
}

/// Copy what an operation may lose. Any failure is returned before the caller changes a byte.
/// A root with nothing to lose gets no backup.
pub(super) fn create(
    root: &Dir,
    operation: &str,
    affected: impl IntoIterator<Item = PathBuf>,
) -> Result<Option<PathBuf>, InstallError> {
    let affected: BTreeSet<PathBuf> = affected.into_iter().collect();
    let mut paths = standard_paths(root)?;
    paths.extend(affected.iter().cloned());

    let mut copies = Vec::new();
    let mut skipped = Vec::new();
    let mut total = 0_u64;
    for path in paths {
        if path
            .extension()
            .is_some_and(|extension| extension == "lock")
            || REGENERABLE.iter().any(|tree| path.starts_with(tree))
        {
            continue;
        }
        if !in_scope(&path) {
            return Err(InstallError::Conflict(format!(
                "cannot back up a path outside Hive's write scope; nothing was changed: {}",
                path.display()
            )));
        }
        let read = match capability_metadata(root, &path) {
            Ok(Err(error)) if error.kind() == io::ErrorKind::NotFound => continue,
            Ok(Err(error)) => Err(io_internal("inspect", &path, error)),
            Ok(Ok(metadata)) if metadata.is_dir() => continue,
            Ok(Ok(_)) => read_optional_regular(root, &path, FILE_LIMIT),
            Err(error) => Err(error),
        };
        let bytes = match read {
            Ok(Some(bytes)) => bytes,
            Ok(None) => continue,
            // A linked or oversized file that this operation does not write is only noted.
            Err(InstallError::Conflict(_)) if !affected.contains(&path) => {
                skipped.push(portable(&path));
                continue;
            }
            Err(error) => {
                return Err(error.with_suffix("; the backup failed, so nothing was changed"))
            }
        };
        total = total.saturating_add(bytes.len() as u64);
        if total > TOTAL_LIMIT {
            return Err(InstallError::Conflict(
                "the recovery backup would exceed 512 MiB; nothing was changed".to_owned(),
            ));
        }
        let executable = file_permissions(root, &path)?.executable;
        copies.push((path, bytes, executable));
    }
    if copies.is_empty() {
        return Ok(None);
    }

    let mut nonce = [0_u8; 8];
    getrandom::fill(&mut nonce)
        .map_err(|error| InstallError::Internal(format!("cannot name the backup: {error}")))?;
    let created_at = unix_seconds()?;
    let directory = Path::new(ROOT).join(format!(
        "{operation}-{created_at}-{}",
        &sha256_digest(&nonce)["sha256:".len()..][..16]
    ));
    let mut entries = Vec::new();
    for (path, bytes, executable) in copies {
        let copy = directory.join("files").join(&path);
        write_private(root, &copy, &bytes)?;
        if read_optional_regular(root, &copy, FILE_LIMIT)?.as_deref() != Some(bytes.as_slice()) {
            return Err(InstallError::Verification(format!(
                "backup copy does not match its source; nothing was changed: {}",
                path.display()
            )));
        }
        entries.push(Entry {
            path: portable(&path),
            digest: sha256_digest(&bytes),
            executable,
        });
    }
    // Written last: a folder without a manifest is an incomplete backup and is never restored.
    write_private(
        root,
        &directory.join(MANIFEST),
        &json_line(&Manifest {
            schema_version: 1,
            operation: operation.to_owned(),
            created_at,
            product_version: env!("CARGO_PKG_VERSION").to_owned(),
            entries,
            skipped,
        })?,
    )?;
    Ok(Some(directory))
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 96
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn read_manifest(root: &Dir, id: &str) -> Result<Option<Manifest>, InstallError> {
    let path = Path::new(ROOT).join(id).join(MANIFEST);
    let Some(bytes) = read_optional_regular(root, &path, FILE_LIMIT)? else {
        return Ok(None);
    };
    let manifest: Manifest = serde_json::from_slice(&bytes)
        .map_err(|_| InstallError::Conflict(format!("backup manifest is damaged: {id}")))?;
    if manifest.schema_version != 1
        || manifest
            .entries
            .iter()
            .any(|entry| !in_scope(Path::new(&entry.path)))
    {
        return Err(InstallError::Conflict(format!(
            "backup manifest names an unsupported format or path: {id}"
        )));
    }
    Ok(Some(manifest))
}

fn success(action: &'static str, code: &'static str, message: &str) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn list(user_root: &Path) -> Result<ActionResult, InstallError> {
    let (_, root) = open_canonical_user_root(user_root)?;
    let base = Path::new(ROOT);
    let mut found = Vec::new();
    match root.open_dir_nofollow(base) {
        Ok(directory) => {
            for entry in directory
                .entries()
                .map_err(|error| io_internal("list", base, error))?
            {
                let entry = entry.map_err(|error| io_internal("list", base, error))?;
                let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                if !valid_id(&id) || !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    continue;
                }
                // Folders that a person made by hand, or that are incomplete, are left alone.
                let Ok(Some(manifest)) = read_manifest(&root, &id) else {
                    continue;
                };
                found.push((manifest.created_at, id, manifest.entries.len()));
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_internal("open", base, error)),
    }
    found.sort_by(|left, right| right.cmp(left));
    let mut result = success(
        "ListHiveBackups",
        "hive.backups-listed",
        "recovery backups were listed; nothing was changed",
    );
    result.data = Some(json!({
        "location": ROOT,
        "items": found
            .into_iter()
            .map(|(created_at, id, count)| json!({"name":id,"created_at":created_at,"count":count}))
            .collect::<Vec<_>>(),
    }));
    Ok(result)
}

struct Change {
    path: PathBuf,
    bytes: Vec<u8>,
    executable: bool,
    current: Option<Vec<u8>>,
}

/// The verified files of one backup that differ from what is on disk now.
fn planned_changes(root: &Dir, id: &str, only: Option<&str>) -> Result<Vec<Change>, InstallError> {
    let manifest = read_manifest(root, id)?.ok_or_else(|| {
        InstallError::Input(format!("no complete recovery backup has this id: {id}"))
    })?;
    if let Some(only) = only {
        if !manifest.entries.iter().any(|entry| entry.path == only) {
            return Err(InstallError::Input(format!(
                "this backup has no such file: {only}"
            )));
        }
    }

    let mut changes = Vec::new();
    for entry in manifest.entries {
        let path = PathBuf::from(&entry.path);
        let selected = only.map_or_else(
            || DEFAULT_RESTORE.contains(&entry.path.as_str()),
            |only| entry.path == only,
        );
        if !selected {
            continue;
        }
        let copy = Path::new(ROOT).join(id).join("files").join(&path);
        let bytes = read_optional_regular(root, &copy, FILE_LIMIT)?
            .filter(|bytes| sha256_digest(bytes) == entry.digest)
            .ok_or_else(|| {
                InstallError::Conflict(format!(
                    "backup copy is missing or changed; nothing was restored: {}",
                    entry.path
                ))
            })?;
        let current = read_optional_regular(root, &path, FILE_LIMIT)?;
        if current.as_deref() != Some(bytes.as_slice()) {
            changes.push(Change {
                path,
                bytes,
                executable: entry.executable,
                current,
            });
        }
    }
    Ok(changes)
}

fn restore(
    user_root: &Path,
    id: &str,
    only: Option<&str>,
    confirm: Option<&str>,
) -> Result<ActionResult, InstallError> {
    if !valid_id(id) {
        return Err(InstallError::Input("invalid backup id".to_owned()));
    }
    let (_, root) = open_canonical_user_root(user_root)?;
    let _lease = confirm
        .map(|_| lock_user_setup(&root).map_err(InstallError::Conflict))
        .transpose()?;
    let changes = planned_changes(&root, id, only)?;
    let preview: Vec<_> = changes
        .iter()
        .map(|change| {
            json!({
                "path": portable(&change.path),
                "before": change.current.as_deref().map(sha256_digest),
                "after": sha256_digest(&change.bytes),
            })
        })
        .collect();
    let approval = sha256_digest(json!([id, only, preview]).to_string().as_bytes());

    let Some(confirm) = confirm else {
        let mut result = success(
            "RestoreHiveBackup",
            "hive.backup-restore-previewed",
            "restore preview only; nothing was changed",
        );
        if !changes.is_empty() {
            let selection = only.map_or_else(String::new, |only| format!(" --path \"{only}\""));
            result.next_action = Some(format!(
                "hive storage backup restore --id {id}{selection} --user-root \"{}\" --confirm {approval}",
                user_root.display()
            ));
        }
        result.data = Some(json!({"items": preview, "approval_digest": approval}));
        return Ok(result);
    };
    if confirm != approval {
        return Err(InstallError::Conflict(
            "files changed after the restore preview; preview again. Nothing was restored"
                .to_owned(),
        ));
    }

    let saved = create(
        &root,
        "restore",
        changes.iter().map(|change| change.path.clone()),
    )?;
    let mut result = success(
        "RestoreHiveBackup",
        "hive.backup-restored",
        "selected backup files were restored; no file was deleted",
    );
    for change in changes {
        let existing = change
            .current
            .as_ref()
            .map(|_| file_permissions(&root, &change.path))
            .transpose()?;
        let permissions = existing.unwrap_or(FilePermissions {
            executable: change.executable,
            unix_mode: if change.executable {
                installed_unix_mode(true)
            } else {
                private_mode()
            },
        });
        write_atomic_with_permissions(
            &root,
            &change.path,
            &change.bytes,
            permissions,
            change.current.as_deref(),
            existing,
        )
        .map_err(|error| match &saved {
            Some(saved) => error.with_suffix(&format!(
                "; the files before this restore were saved at {}",
                portable(saved)
            )),
            None => error,
        })?;
        result.changed_paths.push(portable(&change.path));
    }
    result.data = Some(json!({"backup": saved.as_deref().map(portable)}));
    Ok(result)
}

/// The first step of `hive update`, before settings or projections change.
pub(crate) fn before_update(user_root: &Path) -> Result<Option<PathBuf>, String> {
    open_canonical_user_root(user_root)
        .and_then(|(_, root)| create(&root, "update", Vec::new()))
        .map_err(|error| error.message().to_owned())
}

pub(crate) fn list_command(user_root: &Path) -> ActionResult {
    list(user_root).unwrap_or_else(|error| super::failure("ListHiveBackups", &error))
}

pub(crate) fn restore_command(
    user_root: &Path,
    id: &str,
    only: Option<&str>,
    confirm: Option<&str>,
) -> ActionResult {
    restore(user_root, id, only, confirm)
        .unwrap_or_else(|error| super::failure("RestoreHiveBackup", &error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const PROJECTS: &str = ".hive/config/projects.yml";

    fn seeded() -> (tempfile::TempDir, PathBuf, Dir) {
        let temporary = tempfile::tempdir().unwrap();
        let (user, root) = open_canonical_user_root(temporary.path()).unwrap();
        fs::create_dir_all(user.join(".hive/config")).unwrap();
        fs::write(user.join(PROJECTS), b"sixteen projects\n").unwrap();
        (temporary, user, root)
    }

    fn id_of(directory: &Path) -> String {
        directory.file_name().unwrap().to_str().unwrap().to_owned()
    }

    #[test]
    fn a_confirmed_restore_returns_exact_bytes_and_keeps_what_it_replaced() {
        let (_temporary, user, root) = seeded();
        let saved = create(&root, "install", Vec::new()).unwrap().unwrap();
        assert!(!saved.starts_with(".hive"));
        let id = id_of(&saved);
        // The 0.15.3 loss: the registry emptied and everything below .hive/backups deleted.
        fs::write(user.join(PROJECTS), b"projects: []\n").unwrap();
        let later = user.join(".hive/config/added-later.yml");
        fs::write(&later, b"newer\n").unwrap();

        let preview = restore(&user, &id, None, None).unwrap();
        assert_eq!(preview.changed_paths, Vec::<String>::new());
        assert_eq!(fs::read(user.join(PROJECTS)).unwrap(), b"projects: []\n");
        let approval = preview.data.unwrap()["approval_digest"]
            .as_str()
            .unwrap()
            .to_owned();

        let restored = restore(&user, &id, None, Some(&approval)).unwrap();
        assert_eq!(
            fs::read(user.join(PROJECTS)).unwrap(),
            b"sixteen projects\n"
        );
        assert_eq!(fs::read(&later).unwrap(), b"newer\n");
        let replaced = restored.data.unwrap()["backup"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            fs::read(user.join(replaced).join("files").join(PROJECTS)).unwrap(),
            b"projects: []\n"
        );
    }

    #[test]
    fn a_file_changed_after_preview_is_not_overwritten() {
        let (_temporary, user, root) = seeded();
        let id = id_of(&create(&root, "install", Vec::new()).unwrap().unwrap());
        fs::write(user.join(PROJECTS), b"first edit\n").unwrap();
        let approval = restore(&user, &id, None, None).unwrap().data.unwrap()["approval_digest"]
            .as_str()
            .unwrap()
            .to_owned();
        fs::write(user.join(PROJECTS), b"second edit\n").unwrap();
        assert!(restore(&user, &id, None, Some(&approval)).is_err());
        assert_eq!(fs::read(user.join(PROJECTS)).unwrap(), b"second edit\n");
    }

    #[test]
    fn a_changed_copy_or_an_escaping_manifest_is_refused() {
        let (_temporary, user, root) = seeded();
        let saved = create(&root, "install", Vec::new()).unwrap().unwrap();
        let id = id_of(&saved);
        fs::write(user.join(&saved).join("files").join(PROJECTS), b"forged\n").unwrap();
        assert!(restore(&user, &id, None, None).is_err());

        let manifest = user.join(&saved).join(MANIFEST);
        let original = fs::read_to_string(&manifest).unwrap();
        for path in [
            "../outside.yml",
            ".codex/auth.json",
            ".claude/.credentials.json",
        ] {
            fs::write(&manifest, original.replace(PROJECTS, path)).unwrap();
            assert!(restore(&user, &id, None, None).is_err(), "{path}");
        }
        assert!(restore(&user, "../x", None, None).is_err());
    }

    #[test]
    fn a_default_restore_leaves_state_and_product_files_alone() {
        let (_temporary, user, root) = seeded();
        let stale = [
            ".hive/runtime/usage-guard/halt.json",
            ".hive/claims/old.json",
            ".hive/guides/guide.md",
        ];
        for relative in stale {
            let path = user.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"old\n").unwrap();
        }
        let saved = create(&root, "uninstall", stale.map(PathBuf::from))
            .unwrap()
            .unwrap();
        for relative in stale {
            fs::write(user.join(relative), b"current\n").unwrap();
        }
        let id = id_of(&saved);
        let preview = restore(&user, &id, None, None).unwrap().data.unwrap();
        assert_eq!(preview["items"], json!([]));
        let named = restore(&user, &id, Some(stale[2]), None)
            .unwrap()
            .data
            .unwrap();
        assert_eq!(named["items"][0]["path"], stale[2]);
        assert_eq!(
            tree_files(&root, Path::new(".hive/guides")).unwrap(),
            [Path::new(".hive/guides").join("guide.md")]
        );
    }

    #[test]
    fn sign_in_files_are_never_copied() {
        let (_temporary, user, root) = seeded();
        fs::create_dir_all(user.join(".codex")).unwrap();
        fs::write(user.join(".codex/auth.json"), b"synthetic").unwrap();
        let saved = create(&root, "install", Vec::new()).unwrap().unwrap();
        assert!(!user.join(&saved).join("files/.codex/auth.json").exists());
        assert!(create(&root, "install", [PathBuf::from(".codex/auth.json")]).is_err());
    }

    #[test]
    fn a_backup_that_cannot_be_written_fails_and_an_empty_root_needs_none() {
        let (_temporary, user, root) = seeded();
        fs::write(user.join(ROOT), b"not a folder").unwrap();
        assert!(create(&root, "install", Vec::new()).is_err());
        assert_eq!(
            fs::read(user.join(PROJECTS)).unwrap(),
            b"sixteen projects\n"
        );

        let empty = tempfile::tempdir().unwrap();
        let (_, empty_root) = open_canonical_user_root(empty.path()).unwrap();
        assert_eq!(create(&empty_root, "install", Vec::new()).unwrap(), None);
    }
}
