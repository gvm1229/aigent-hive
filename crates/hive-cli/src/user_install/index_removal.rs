//! Exact rollback for a confirmed Wiki disable; never reconstruct from project content.
use super::{
    file_permissions, persist_backup, read_optional_regular, remove_regular_if_exists,
    sha256_digest, validate_relative, write_atomic_with_permissions, FilePermissions, InstallError,
    UserArguments, UserTransaction, MAX_DERIVED_INDEX_BYTES, MAX_USER_FILE_BYTES,
};
use cap_fs_ext::{DirExt, MetadataExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

const PATHS: [&str; 8] = hive_wiki::shared::SHARED_DERIVED_RELATIVES;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RemovalBackup {
    schema_version: u32,
    root_digest: String,
    root_identity: (u64, u64),
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    digest: Option<String>,
    permissions: Option<FilePermissions>,
}

fn maximum(path: &str) -> u64 {
    if path.starts_with(".hive/index/hive.sqlite3") {
        MAX_DERIVED_INDEX_BYTES
    } else {
        MAX_USER_FILE_BYTES
    }
}

fn binding(arguments: &UserArguments) -> Result<(String, (u64, u64)), InstallError> {
    let canonical = hive_wiki::shared::canonical_root(&arguments.user_root)
        .map_err(|error| InstallError::Conflict(error.to_string()))?;
    let metadata = arguments
        .root_cap
        .dir_metadata()
        .map_err(|error| InstallError::Conflict(error.to_string()))?;
    Ok((
        sha256_digest(canonical.to_string_lossy().as_bytes()),
        (metadata.dev(), metadata.ino()),
    ))
}

fn payload(backup: &Path, path: &str) -> PathBuf {
    backup.join("index-files").join(path)
}

fn validate_directory(arguments: &UserArguments, backup: &Path) -> Result<(), InstallError> {
    validate_relative(backup)?;
    let parent = Path::new(".hive/backups/user-install").join(arguments.host.as_str());
    if !backup.starts_with(&parent)
        || backup.components().count() != parent.components().count() + 1
    {
        return Err(InstallError::Conflict(
            "index backup is outside this host transaction".into(),
        ));
    }
    Ok(())
}

fn validate(
    arguments: &UserArguments,
    backup: &Path,
    snapshot: &RemovalBackup,
) -> Result<(), InstallError> {
    validate_directory(arguments, backup)?;
    let (digest, identity) = binding(arguments)?;
    let paths: BTreeSet<_> = snapshot
        .entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    if snapshot.schema_version != 1
        || snapshot.root_digest != digest
        || snapshot.root_identity != identity
        || snapshot.entries.len() != PATHS.len()
        || paths != PATHS.into_iter().collect()
        || snapshot.entries.iter().any(|entry| {
            entry.digest.is_some() != entry.permissions.is_some()
                || entry
                    .digest
                    .as_ref()
                    .is_some_and(|digest| !crate::is_sha256_digest(digest))
                || entry.permissions.is_some_and(|permissions| {
                    permissions.unix_mode.is_some_and(|mode| mode > 0o7777)
                        || (cfg!(unix) && permissions.unix_mode.is_none())
                })
        })
    {
        return Err(InstallError::Conflict(
            "invalid index removal recovery binding".into(),
        ));
    }
    Ok(())
}

fn capture(arguments: &UserArguments, backup: &Path) -> Result<RemovalBackup, InstallError> {
    validate_directory(arguments, backup)?;
    let mut entries = Vec::new();
    for path in PATHS {
        let original = read_optional_regular(&arguments.root_cap, Path::new(path), maximum(path))?;
        let permissions = original
            .as_ref()
            .map(|_| file_permissions(&arguments.root_cap, Path::new(path)))
            .transpose()?;
        let relative = payload(backup, path);
        let retained = read_optional_regular(&arguments.root_cap, &relative, maximum(path))?;
        if retained
            .as_ref()
            .is_some_and(|retained| Some(retained) != original.as_ref())
        {
            return Err(InstallError::Conflict(
                "retained index backup changed before capture".into(),
            ));
        }
        if let Some(bytes) = &original {
            let retained_permissions = retained
                .as_ref()
                .map(|_| file_permissions(&arguments.root_cap, &relative))
                .transpose()?;
            let mut private = super::installed_file_permissions(false);
            if cfg!(unix) {
                private.unix_mode = Some(0o600);
            }
            write_atomic_with_permissions(
                &arguments.root_cap,
                &relative,
                bytes,
                private,
                retained.as_deref(),
                retained_permissions,
            )?;
        }
        entries.push(Entry {
            path: path.into(),
            digest: original.as_deref().map(sha256_digest),
            permissions,
        });
    }
    let (root_digest, root_identity) = binding(arguments)?;
    Ok(RemovalBackup {
        schema_version: 1,
        root_digest,
        root_identity,
        entries,
    })
}

fn checked_original(
    arguments: &UserArguments,
    backup: &Path,
    entry: &Entry,
) -> Result<Option<Vec<u8>>, InstallError> {
    let original = read_optional_regular(
        &arguments.root_cap,
        &payload(backup, &entry.path),
        maximum(&entry.path),
    )?;
    if original.as_deref().map(sha256_digest) != entry.digest {
        return Err(InstallError::Conflict(
            "index recovery payload changed".into(),
        ));
    }
    Ok(original)
}

fn check_current(
    arguments: &UserArguments,
    entry: &Entry,
) -> Result<Option<Vec<u8>>, InstallError> {
    let current = read_optional_regular(
        &arguments.root_cap,
        Path::new(&entry.path),
        maximum(&entry.path),
    )?;
    if current.is_some()
        && (current.as_deref().map(sha256_digest) != entry.digest
            || Some(file_permissions(
                &arguments.root_cap,
                Path::new(&entry.path),
            )?) != entry.permissions)
    {
        return Err(InstallError::Conflict(format!(
            "index file changed during removal recovery: {}",
            entry.path
        )));
    }
    Ok(current)
}

fn reconcile_claim(
    arguments: &UserArguments,
    entry: &Entry,
    recover: bool,
) -> Result<(), InstallError> {
    let relative = Path::new(&entry.path);
    let Some((parent, _)) = super::capability_parent(&arguments.root_cap, relative, false)? else {
        return Ok(());
    };
    let name = super::claim_name(relative);
    let claim = match parent.open_dir_nofollow(&name) {
        Ok(claim) => claim,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(InstallError::Conflict(error.to_string())),
    };
    let mut retained = false;
    for item in claim
        .entries()
        .map_err(|error| InstallError::Conflict(error.to_string()))?
    {
        if item
            .map_err(|error| InstallError::Conflict(error.to_string()))?
            .file_name()
            != "claimed.bin"
        {
            return Err(InstallError::Conflict(
                "index removal claim contains an unrecognized object".into(),
            ));
        }
        retained = true;
    }
    if !retained {
        drop(claim);
        if recover {
            parent
                .remove_dir(name)
                .map_err(|error| InstallError::Conflict(error.to_string()))?;
        }
        return Ok(());
    }
    let bytes = read_optional_regular(&claim, Path::new("claimed.bin"), maximum(&entry.path))?;
    if bytes.as_deref().map(sha256_digest) != entry.digest
        || entry.digest.is_none()
        || Some(file_permissions(&claim, Path::new("claimed.bin"))?) != entry.permissions
    {
        return Err(InstallError::Conflict(
            "retained index removal bytes or permissions changed".into(),
        ));
    }
    drop(claim);
    if recover {
        let permissions = entry.permissions.expect("verified original");
        let original = super::UserBackupEntry {
            path: entry.path.clone(),
            existed: true,
            digest: entry.digest.clone(),
            installed_digest: None,
            installed_executable: None,
            installed_unix_mode: None,
            executable: permissions.executable,
            unix_mode: permissions.unix_mode,
        };
        super::reconcile_retained_claim_bounded(
            &arguments.root_cap,
            relative,
            &original,
            maximum(&entry.path),
        )?;
    }
    Ok(())
}

fn remove(
    arguments: &UserArguments,
    backup: &Path,
    snapshot: &RemovalBackup,
) -> Result<(), InstallError> {
    validate(arguments, backup, snapshot)?;
    for entry in &snapshot.entries {
        checked_original(arguments, backup, entry)?;
        check_current(arguments, entry)?;
        reconcile_claim(arguments, entry, false)?;
    }
    for entry in &snapshot.entries {
        reconcile_claim(arguments, entry, true)?;
    }
    for (index, entry) in snapshot.entries.iter().enumerate() {
        interrupt_fixture(index);
        let current = check_current(arguments, entry)?;
        remove_regular_if_exists(
            &arguments.root_cap,
            Path::new(&entry.path),
            current.as_deref(),
            current.as_ref().and(entry.permissions),
        )?;
    }
    interrupt_fixture(snapshot.entries.len());
    for entry in &snapshot.entries {
        if check_current(arguments, entry)?.is_some() {
            return Err(InstallError::Conflict(
                "index file appeared after removal".into(),
            ));
        }
    }
    Ok(())
}

fn interrupt_fixture(index: usize) {
    #[cfg(debug_assertions)]
    if std::env::var("HIVE_USER_INSTALL_INDEX_INTERRUPT_AFTER")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        == Some(index)
    {
        std::process::exit(88);
    }
    #[cfg(not(debug_assertions))]
    let _ = index;
}

pub(super) fn apply(
    arguments: &UserArguments,
    transaction: &mut UserTransaction,
) -> Result<(), InstallError> {
    let store = hive_wiki::store::RagStore::open_bound(&arguments.user_root, &arguments.root_cap)
        .map_err(|error| InstallError::Conflict(error.to_string()))?;
    let _lock = store
        .acquire_authorization_lock()
        .map_err(|error| InstallError::Conflict(error.to_string()))?;
    if transaction.backup.index_removal.is_none() {
        transaction.backup.index_removal = Some(capture(arguments, &transaction.backup_relative)?);
        transaction.backup.schema_version = 3;
        persist_backup(arguments, &transaction.backup_relative, &transaction.backup)?;
    }
    remove(
        arguments,
        &transaction.backup_relative,
        transaction
            .backup
            .index_removal
            .as_ref()
            .expect("captured removal"),
    )
}

pub(super) fn restore(
    arguments: &UserArguments,
    backup: &Path,
    snapshot: &RemovalBackup,
) -> Result<(), InstallError> {
    let store = hive_wiki::store::RagStore::open_bound(&arguments.user_root, &arguments.root_cap)
        .map_err(|error| InstallError::Conflict(error.to_string()))?;
    let _lock = store
        .acquire_authorization_lock()
        .map_err(|error| InstallError::Conflict(error.to_string()))?;
    validate(arguments, backup, snapshot)?;
    for entry in &snapshot.entries {
        checked_original(arguments, backup, entry)?;
        check_current(arguments, entry)?;
        reconcile_claim(arguments, entry, false)?;
    }
    for entry in &snapshot.entries {
        reconcile_claim(arguments, entry, true)?;
    }
    for entry in &snapshot.entries {
        let current = check_current(arguments, entry)?;
        if current.is_none() {
            if let Some(original) = checked_original(arguments, backup, entry)? {
                write_atomic_with_permissions(
                    &arguments.root_cap,
                    Path::new(&entry.path),
                    &original,
                    entry.permissions.expect("original permissions"),
                    None,
                    None,
                )?;
            }
        }
    }
    for entry in &snapshot.entries {
        if check_current(arguments, entry)?
            .as_deref()
            .map(sha256_digest)
            != entry.digest
        {
            return Err(InstallError::Conflict(
                "index restoration is incomplete".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cap_std::ambient_authority;
    use cap_std::fs::Dir;

    fn fixture() -> (tempfile::TempDir, UserArguments, PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let arguments = UserArguments {
            host: super::super::UserHost::Codex,
            mode: super::super::UserMode::Apply,
            user_root: root.path().canonicalize().unwrap(),
            root_cap: Dir::open_ambient_dir(root.path(), ambient_authority()).unwrap(),
            setup_override: None,
        };
        for path in [PATHS[0], PATHS[4], PATHS[5]] {
            super::super::replace_user_setup_file(
                &arguments.root_cap,
                Path::new(path),
                None,
                Some(path.as_bytes()),
            )
            .unwrap();
        }
        (
            root,
            arguments,
            PathBuf::from(".hive/backups/user-install/codex/index-test"),
        )
    }

    #[test]
    fn removal_restores_exact_index_bytes_and_is_repeatable_without_project_reads() {
        let (_root, arguments, backup) = fixture();
        let snapshot = capture(&arguments, &backup).unwrap();
        remove(&arguments, &backup, &snapshot).unwrap();
        for path in PATHS {
            assert!(
                read_optional_regular(&arguments.root_cap, Path::new(path), maximum(path))
                    .unwrap()
                    .is_none()
            );
        }
        let (parent, _) =
            super::super::capability_parent(&arguments.root_cap, Path::new(PATHS[0]), true)
                .unwrap()
                .unwrap();
        parent
            .create_dir(super::super::claim_name(Path::new(PATHS[0])))
            .unwrap();
        restore(&arguments, &backup, &snapshot).unwrap();
        restore(&arguments, &backup, &snapshot).unwrap();
        for entry in &snapshot.entries {
            assert_eq!(
                check_current(&arguments, entry)
                    .unwrap()
                    .as_deref()
                    .map(sha256_digest),
                entry.digest
            );
        }
    }

    #[test]
    fn changed_live_index_or_recovery_payload_is_preserved_and_rejected() {
        let (root, arguments, backup) = fixture();
        let snapshot = capture(&arguments, &backup).unwrap();
        std::fs::write(root.path().join(PATHS[0]), b"new external index").unwrap();
        assert!(remove(&arguments, &backup, &snapshot).is_err());
        assert_eq!(
            std::fs::read(root.path().join(PATHS[4])).unwrap(),
            PATHS[4].as_bytes()
        );
        assert!(restore(&arguments, &backup, &snapshot).is_err());
        assert_eq!(
            std::fs::read(root.path().join(PATHS[0])).unwrap(),
            b"new external index"
        );
        std::fs::write(
            root.path().join(payload(&backup, PATHS[0])),
            b"tampered backup",
        )
        .unwrap();
        assert!(restore(&arguments, &backup, &snapshot).is_err());
    }

    #[test]
    fn removal_recovery_restores_a_claimed_index_larger_than_the_harness_file_limit() {
        let (root, arguments, backup) = fixture();
        let bytes = vec![b'x'; usize::try_from(MAX_USER_FILE_BYTES).unwrap() + 1];
        std::fs::write(root.path().join(PATHS[0]), &bytes).unwrap();
        let snapshot = capture(&arguments, &backup).unwrap();
        super::super::interrupt_setup_remove(&arguments.root_cap, Path::new(PATHS[0]), &bytes);
        assert!(!root.path().join(PATHS[0]).exists());
        restore(&arguments, &backup, &snapshot).unwrap();
        assert_eq!(
            sha256_digest(&std::fs::read(root.path().join(PATHS[0])).unwrap()),
            sha256_digest(&bytes)
        );
    }

    #[test]
    fn recovery_cannot_expand_paths_or_move_to_another_user_root() {
        let (_root, arguments, backup) = fixture();
        let mut snapshot = capture(&arguments, &backup).unwrap();
        let (_other, other, _) = fixture();
        assert!(restore(&other, &backup, &snapshot).is_err());
        snapshot.entries[0].path = "foreign.txt".into();
        assert!(restore(&arguments, &backup, &snapshot).is_err());
    }
}
