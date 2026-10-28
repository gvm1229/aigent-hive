//! Finish only the exact retired image recorded by the Windows direct installer.
use super::{
    discover_direct, parse_package_version, sha256_digest, MAX_BINARY_BYTES, MAX_MANIFEST_BYTES,
};
use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    thread,
    time::{Duration, Instant},
};

const RECORD: &str = "share/aigent-hive/replacement-pending.json";
const RECEIPT: &str = "share/aigent-hive/install-receipt.json";
const INSTALL_PENDING: &str = "share/aigent-hive/install-receipt.pending.json";
const LIMIT: u64 = 32 * 1024;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: u8,
    prefix_identity: String,
    backup_name: String,
    before_sha256: String,
    after_sha256: String,
    before_package_version: String,
    after_package_version: String,
}

fn prefix_identity(root: &Dir) -> Result<String, String> {
    let metadata = root
        .dir_metadata()
        .map_err(|_| "direct prefix identity unavailable")?;
    serde_json::to_vec(&("direct-replacement-root-v1", metadata.dev(), metadata.ino()))
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|_| "cannot encode direct prefix identity".into())
}

pub(crate) fn run_scope(arguments: &[String]) -> ExitCode {
    if !cfg!(windows) || arguments.len() != 2 || arguments[0] != "--prefix" {
        return ExitCode::from(2);
    }
    let outcome = crate::run::PinnedTarget::open_usage(Path::new(&arguments[1]))
        .map_err(|e| e.message().to_owned())
        .and_then(|root| prefix_identity(root.target_dir()));
    match outcome {
        Ok(identity) => {
            println!("{}", json!({"prefix_identity":identity}));
            ExitCode::SUCCESS
        }
        Err(_) => ExitCode::from(2),
    }
}

struct Pending {
    root: Dir,
    prefix: PathBuf,
    executable: PathBuf,
    record: Record,
    bytes: Vec<u8>,
}

fn read(root: &Dir, path: &Path, limit: u64) -> Result<Option<Vec<u8>>, String> {
    crate::user_install::read_user_setup_file(root, path, limit)
}

fn remove(root: &Dir, path: &Path, bytes: &[u8]) -> Result<(), String> {
    crate::user_install::replace_user_setup_file(root, path, Some(bytes), None)
}

fn decode(bytes: &[u8]) -> Result<Record, String> {
    let record: Record = serde_json::from_slice(bytes).map_err(|_| "invalid replacement record")?;
    let nonce = record
        .backup_name
        .strip_prefix(".hive-replace-backup-")
        .ok_or("unowned replacement backup name")?;
    if record.schema_version != 1
        || !crate::is_sha256_digest(&record.prefix_identity)
        || nonce.len() != 32
        || !nonce.bytes().all(|b| b.is_ascii_hexdigit())
        || !crate::is_sha256_digest(&record.before_sha256)
        || !crate::is_sha256_digest(&record.after_sha256)
        || record.before_sha256 == record.after_sha256
    {
        return Err("invalid replacement scope or digest".into());
    }
    parse_package_version(&record.before_package_version)?;
    parse_package_version(&record.after_package_version)?;
    Ok(record)
}

fn matches_receipt(bytes: &[u8], package: &str, digest: &str) -> Result<(), String> {
    let receipt: super::DirectReceipt =
        serde_json::from_slice(bytes).map_err(|_| "invalid replacement install receipt")?;
    let version = parse_package_version(package)?;
    if receipt.schema_version != 1
        || receipt.owner != "direct"
        || receipt.product != "aigent-hive"
        || receipt.package_version != package
        || receipt.version != version.product.to_string()
        || receipt.artifact_sha256 != digest
    {
        return Err("replacement receipt differs; files preserved".into());
    }
    Ok(())
}

fn recover_claim(
    root: &Dir,
    path: &Path,
    limit: u64,
    validate: impl Fn(&[u8], bool) -> Result<(), String>,
) -> Result<(), String> {
    if let Some(claim) = crate::user_install::inspect_setup_write_claim(root, path, limit)? {
        if let Some(bytes) = &claim.before {
            validate(bytes, false)?;
        }
        if let Some(bytes) = &claim.staged {
            validate(bytes, true)?;
        }
        let before = claim
            .before
            .as_deref()
            .map_or_else(|| "absent".into(), sha256_digest);
        let after = claim
            .staged
            .as_deref()
            .map_or_else(|| "absent".into(), sha256_digest);
        if path == Path::new(RECORD) {
            crate::user_install::recover_user_setup_record(
                root,
                path,
                &before,
                &after,
                claim.before_mode,
            )?;
        } else {
            crate::user_install::recover_user_setup_file(
                root,
                path,
                &before,
                &after,
                claim.before_mode,
            )?;
        }
    }
    Ok(())
}

impl Pending {
    fn backup(&self) -> PathBuf {
        Path::new("bin").join(&self.record.backup_name)
    }

    fn view(&self, state: &str) -> Value {
        json!({"state":state,"scope":"direct-install-prefix","prefix":self.prefix,
            "backup":self.backup(),"record":RECORD,"record_digest":sha256_digest(&self.bytes),
            "from_version":self.record.before_package_version,"to_version":self.record.after_package_version})
    }

    fn verify(&self) -> Result<(), String> {
        let current = crate::run::PinnedTarget::open_usage(&self.prefix)
            .map_err(|error| error.message().to_owned())?;
        if prefix_identity(current.target_dir())? != self.record.prefix_identity {
            return Err("direct replacement prefix changed; preserved".into());
        }
        if read(&self.root, Path::new(RECORD), LIMIT)?.as_deref() != Some(self.bytes.as_slice()) {
            return Err("replacement record changed; files preserved".into());
        }
        let receipt = read(&self.root, Path::new(RECEIPT), MAX_MANIFEST_BYTES)?
            .ok_or("replacement receipt missing")?;
        matches_receipt(
            &receipt,
            &self.record.after_package_version,
            &self.record.after_sha256,
        )?;
        let package = parse_package_version(&self.record.after_package_version)?;
        discover_direct(&self.executable, &self.prefix, &receipt, package.product)?;
        Ok(())
    }
}

fn peek(root: &Dir, path: &Path, limit: u64) -> Result<Option<Vec<u8>>, String> {
    if let Some(bytes) = read(root, path, limit)? {
        return Ok(Some(bytes));
    }
    Ok(
        crate::user_install::inspect_setup_write_claim(root, path, limit)?
            .and_then(|claim| claim.before),
    )
}

fn authenticate_snapshot(root: &Dir, record: &Record) -> Result<(), String> {
    if prefix_identity(root)? != record.prefix_identity {
        return Err("direct replacement belongs to another prefix".into());
    }
    let active = read(root, Path::new("bin/hive.exe"), MAX_BINARY_BYTES)?
        .ok_or("replacement executable missing")?;
    let receipt =
        peek(root, Path::new(RECEIPT), MAX_MANIFEST_BYTES)?.ok_or("replacement receipt missing")?;
    let pending = peek(root, Path::new(INSTALL_PENDING), MAX_MANIFEST_BYTES)?;
    let backup = peek(
        root,
        &Path::new("bin").join(&record.backup_name),
        MAX_BINARY_BYTES,
    )?;
    if backup
        .as_ref()
        .is_some_and(|bytes| sha256_digest(bytes) != record.before_sha256)
    {
        return Err("replacement backup changed; preserved".into());
    }
    let active_digest = sha256_digest(&active);
    if active_digest == record.before_sha256 && backup.is_none() {
        matches_receipt(
            &receipt,
            &record.before_package_version,
            &record.before_sha256,
        )?;
    } else if active_digest == record.after_sha256 {
        if matches_receipt(
            &receipt,
            &record.after_package_version,
            &record.after_sha256,
        )
        .is_err()
        {
            matches_receipt(
                &receipt,
                &record.before_package_version,
                &record.before_sha256,
            )?;
            matches_receipt(
                pending
                    .as_ref()
                    .ok_or("replacement publication is incomplete")?,
                &record.after_package_version,
                &record.after_sha256,
            )?;
        }
    } else {
        return Err("replacement executable changed; preserved".into());
    }
    if let Some(bytes) = pending {
        matches_receipt(&bytes, &record.after_package_version, &record.after_sha256)?;
    }
    Ok(())
}

fn load(executable: &Path, confirmation: Option<&str>) -> Result<Option<Pending>, String> {
    if executable.file_name().and_then(|s| s.to_str()) != Some("hive.exe")
        || executable
            .parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            != Some("bin")
    {
        return Ok(None);
    }
    let executable = executable
        .canonicalize()
        .map_err(|_| "cannot resolve replacement executable")?;
    let prefix = executable
        .parent()
        .and_then(Path::parent)
        .ok_or("replacement prefix missing")?
        .to_path_buf();
    let root = crate::user_install::open_user_root_for_setup(&prefix)?;
    let Some(candidate) = peek(&root, Path::new(RECORD), LIMIT)? else {
        return Ok(None);
    };
    let record = decode(&candidate)?;
    if confirmation.is_some_and(|expected| expected != sha256_digest(&candidate)) {
        return Err("replacement confirmation differs".into());
    }
    authenticate_snapshot(&root, &record)?;
    // A retained CAS record can only restore the same typed installer intent.
    recover_claim(&root, Path::new(RECORD), LIMIT, |bytes, staged| {
        if staged {
            return Err("unexpected replacement record publication claim".into());
        }
        decode(bytes)?;
        if confirmation.is_some_and(|expected| expected != sha256_digest(bytes)) {
            return Err("replacement confirmation differs".into());
        }
        Ok(())
    })?;
    let Some(bytes) = read(&root, Path::new(RECORD), LIMIT)? else {
        return Ok(None);
    };
    if confirmation.is_some_and(|expected| expected != sha256_digest(&bytes)) {
        return Err("replacement confirmation differs".into());
    }
    let record = decode(&bytes)?;
    let backup = Path::new("bin").join(&record.backup_name);
    recover_claim(&root, &backup, MAX_BINARY_BYTES, |bytes, staged| {
        if staged || sha256_digest(bytes) != record.before_sha256 {
            return Err("retained replacement backup differs".into());
        }
        Ok(())
    })?;
    recover_claim(
        &root,
        Path::new(RECEIPT),
        MAX_MANIFEST_BYTES,
        |bytes, staged| {
            if staged {
                matches_receipt(bytes, &record.after_package_version, &record.after_sha256)
            } else {
                matches_receipt(bytes, &record.before_package_version, &record.before_sha256)
            }
        },
    )?;
    recover_claim(
        &root,
        Path::new(INSTALL_PENDING),
        MAX_MANIFEST_BYTES,
        |bytes, staged| {
            if staged {
                return Err("unexpected pending install receipt publication claim".into());
            }
            matches_receipt(bytes, &record.after_package_version, &record.after_sha256)
        },
    )?;
    finish_publication(Pending {
        root,
        prefix,
        executable,
        record,
        bytes,
    })
}

fn finish_publication(transaction: Pending) -> Result<Option<Pending>, String> {
    let root = &transaction.root;
    let record = &transaction.record;
    let bytes = &transaction.bytes;
    let backup = transaction.backup();
    let active = read(root, Path::new("bin/hive.exe"), MAX_BINARY_BYTES)?
        .ok_or("replacement executable missing")?;
    let receipt = read(root, Path::new(RECEIPT), MAX_MANIFEST_BYTES)?
        .ok_or("replacement install receipt missing")?;
    let pending = read(root, Path::new(INSTALL_PENDING), MAX_MANIFEST_BYTES)?;
    if sha256_digest(&active) == record.before_sha256
        && read(root, &backup, MAX_BINARY_BYTES)?.is_none()
    {
        matches_receipt(
            &receipt,
            &record.before_package_version,
            &record.before_sha256,
        )?;
        if let Some(pending) = pending {
            matches_receipt(
                &pending,
                &record.after_package_version,
                &record.after_sha256,
            )?;
            remove(root, Path::new(INSTALL_PENDING), &pending)?;
        }
        remove(root, Path::new(RECORD), bytes)?;
        return Ok(None);
    }
    if sha256_digest(&active) != record.after_sha256 {
        return Err("active replacement changed; files preserved".into());
    }
    if matches_receipt(
        &receipt,
        &record.after_package_version,
        &record.after_sha256,
    )
    .is_err()
    {
        matches_receipt(
            &receipt,
            &record.before_package_version,
            &record.before_sha256,
        )?;
        let pending = pending
            .as_ref()
            .ok_or("replacement install publication is incomplete")?;
        matches_receipt(pending, &record.after_package_version, &record.after_sha256)?;
        crate::user_install::replace_user_setup_file(
            root,
            Path::new(RECEIPT),
            Some(&receipt),
            Some(pending),
        )?;
        interruption("receipt");
    }
    if let Some(pending) = pending {
        matches_receipt(
            &pending,
            &record.after_package_version,
            &record.after_sha256,
        )?;
        remove(root, Path::new(INSTALL_PENDING), &pending)?;
    }
    transaction.verify()?;
    Ok(Some(transaction))
}

pub(super) fn finish(
    executable: &Path,
    confirmation: Option<&str>,
    wait: bool,
) -> Result<Option<Value>, String> {
    if !cfg!(windows) {
        return Ok(None);
    }
    finish_loaded(load(executable, confirmation)?, wait)
}

fn finish_loaded(pending: Option<Pending>, wait: bool) -> Result<Option<Value>, String> {
    let Some(pending) = pending else {
        return Ok(None);
    };
    let backup = pending.backup();
    if let Some(bytes) = read(&pending.root, &backup, MAX_BINARY_BYTES)? {
        if sha256_digest(&bytes) != pending.record.before_sha256 {
            return Err("replacement backup changed; preserved".into());
        }
        let until = Instant::now() + Duration::from_secs(30);
        loop {
            let mut options = OpenOptions::new();
            options.write(true).follow(FollowSymlinks::No);
            match pending.root.open_with(&backup, &options) {
                Ok(file) => {
                    drop(file);
                    break;
                }
                Err(error) if image_in_use(&error) => {
                    if !wait {
                        return Ok(Some(pending.view("after-process-exit")));
                    }
                    if Instant::now() >= until {
                        return Err(
                            "retired image is still in use or not writable; preserved".into()
                        );
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(format!("cannot access exact retired image: {error}")),
            }
        }
        pending.verify()?;
        remove(&pending.root, &backup, &bytes)?;
        interruption("backup");
    }
    pending.verify()?;
    remove(&pending.root, Path::new(RECORD), &pending.bytes)?;
    Ok(Some(pending.view("complete")))
}

fn image_in_use(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::PermissionDenied
        || (cfg!(windows) && matches!(error.raw_os_error(), Some(32 | 33)))
}

fn interruption(point: &str) {
    #[cfg(all(debug_assertions, not(test)))]
    if std::env::var("HIVE_TEST_DIRECT_REPLACEMENT_EXIT_AFTER").as_deref() == Ok(point) {
        std::process::exit(91);
    }
    #[cfg(any(not(debug_assertions), test))]
    let _ = point;
}

pub(super) fn spawn(executable: &Path, view: &Value) -> Result<(), String> {
    let confirmation = view["record_digest"]
        .as_str()
        .filter(|s| crate::is_sha256_digest(s))
        .ok_or("invalid replacement cleanup confirmation")?;
    let mut command = Command::new(executable);
    command
        .args(["__cleanup-direct-replacement", "--confirm", confirmation])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
        .spawn()
        .map_err(|_| "cannot start retired-image cleanup; resume hive update")?;
    Ok(())
}

pub(crate) fn run_worker(arguments: &[String]) -> ExitCode {
    if !cfg!(windows)
        || arguments.len() != 2
        || arguments[0] != "--confirm"
        || !crate::is_sha256_digest(&arguments[1])
    {
        return ExitCode::from(2);
    }
    let outcome = std::env::current_exe()
        .map_err(|e| e.to_string())
        .and_then(|executable| finish(&executable, Some(&arguments[1]), true));
    if matches!(outcome, Ok(Some(value)) if value["state"] == "complete") {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn only_permission_and_native_sharing_locks_are_pending() {
        assert!(image_in_use(&io::Error::from(
            io::ErrorKind::PermissionDenied
        )));
        for code in [32, 33] {
            assert_eq!(
                image_in_use(&io::Error::from_raw_os_error(code)),
                cfg!(windows)
            );
        }
        for kind in [
            io::ErrorKind::NotFound,
            io::ErrorKind::InvalidInput,
            io::ErrorKind::UnexpectedEof,
        ] {
            assert!(!image_in_use(&io::Error::from(kind)));
        }
    }

    fn receipt(package: &str, digest: &str) -> Vec<u8> {
        serde_json::to_vec(
            &json!({"schema_version":1,"owner":"direct","product":"aigent-hive",
            "version":parse_package_version(package).unwrap().product.to_string(),
            "package_version":package,"artifact_sha256":digest}),
        )
        .unwrap()
    }

    fn fixture() -> (tempfile::TempDir, PathBuf, Record) {
        let temp = crate::test_tempdir();
        let executable = temp.path().join("bin/hive.exe");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::create_dir_all(temp.path().join("share/aigent-hive")).unwrap();
        let record = Record {
            schema_version: 1,
            prefix_identity: prefix_identity(
                &crate::user_install::open_user_root_for_setup(temp.path()).unwrap(),
            )
            .unwrap(),
            backup_name: format!(".hive-replace-backup-{}", "a".repeat(32)),
            before_sha256: sha256_digest(b"old image"),
            after_sha256: sha256_digest(b"new image"),
            before_package_version: "0.13.1".into(),
            after_package_version: env!("CARGO_PKG_VERSION").into(),
        };
        fs::write(&executable, b"new image").unwrap();
        fs::write(
            temp.path().join("bin").join(&record.backup_name),
            b"old image",
        )
        .unwrap();
        fs::write(
            temp.path().join(RECEIPT),
            receipt(&record.after_package_version, &record.after_sha256),
        )
        .unwrap();
        fs::write(
            temp.path().join(RECORD),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
        fs::write(
            temp.path().join("bin/neighbor.txt"),
            b"preserve user content",
        )
        .unwrap();
        (temp, executable, record)
    }

    #[test]
    fn exact_retired_image_is_removed_without_touching_the_active_program_or_neighbors() {
        let (temp, executable, record) = fixture();
        let result = finish_loaded(load(&executable, None).unwrap(), false)
            .unwrap()
            .unwrap();
        assert_eq!(result["state"], "complete");
        assert!(!temp.path().join("bin").join(record.backup_name).exists());
        assert!(!temp.path().join(RECORD).exists());
        assert_eq!(fs::read(executable.clone()).unwrap(), b"new image");
        assert_eq!(
            fs::read(temp.path().join("bin/neighbor.txt")).unwrap(),
            b"preserve user content"
        );
        assert!(load(&executable, None).unwrap().is_none());
    }

    #[test]
    fn changed_backup_scope_active_image_or_confirmation_is_preserved() {
        for change in ["backup", "active", "scope", "confirmation", "root"] {
            let (temp, executable, mut record) = fixture();
            if change == "backup" {
                fs::write(
                    temp.path().join("bin").join(&record.backup_name),
                    b"changed",
                )
                .unwrap();
            }
            if change == "active" {
                fs::write(&executable, b"changed").unwrap();
            }
            if change == "scope" {
                record.backup_name = "../neighbor.txt".into();
                fs::write(
                    temp.path().join(RECORD),
                    serde_json::to_vec(&record).unwrap(),
                )
                .unwrap();
            }
            if change == "root" {
                record.prefix_identity = sha256_digest(b"another prefix");
                fs::write(
                    temp.path().join(RECORD),
                    serde_json::to_vec(&record).unwrap(),
                )
                .unwrap();
            }
            let record_bytes = fs::read(temp.path().join(RECORD)).unwrap();
            let confirmation = (change == "confirmation").then_some("sha256:wrong");
            assert!(load(&executable, confirmation).is_err(), "{change}");
            assert_eq!(fs::read(temp.path().join(RECORD)).unwrap(), record_bytes);
            assert_eq!(
                fs::read(temp.path().join("bin/neighbor.txt")).unwrap(),
                b"preserve user content"
            );
        }
    }

    #[test]
    fn published_core_finishes_its_exact_pending_install_receipt_before_cleanup() {
        let (temp, executable, record) = fixture();
        let after = fs::read(temp.path().join(RECEIPT)).unwrap();
        fs::write(temp.path().join(INSTALL_PENDING), &after).unwrap();
        fs::write(
            temp.path().join(RECEIPT),
            receipt(&record.before_package_version, &record.before_sha256),
        )
        .unwrap();
        finish_loaded(load(&executable, None).unwrap(), false).unwrap();
        assert_eq!(fs::read(temp.path().join(RECEIPT)).unwrap(), after);
        assert!(!temp.path().join(INSTALL_PENDING).exists());
        assert!(!temp.path().join(RECORD).exists());
    }

    #[test]
    fn an_unpublished_replacement_retires_only_its_preparation_records() {
        let (temp, executable, record) = fixture();
        fs::write(
            temp.path().join(INSTALL_PENDING),
            fs::read(temp.path().join(RECEIPT)).unwrap(),
        )
        .unwrap();
        fs::write(
            temp.path().join(RECEIPT),
            receipt(&record.before_package_version, &record.before_sha256),
        )
        .unwrap();
        fs::write(&executable, b"old image").unwrap();
        fs::remove_file(temp.path().join("bin").join(record.backup_name)).unwrap();
        assert!(load(&executable, None).unwrap().is_none());
        assert_eq!(fs::read(&executable).unwrap(), b"old image");
        assert!(!temp.path().join(INSTALL_PENDING).exists());
        assert!(!temp.path().join(RECORD).exists());
    }

    #[test]
    fn interrupted_backup_and_record_removal_reuse_the_existing_file_recovery() {
        for record_removal in [false, true] {
            let (temp, executable, record) = fixture();
            let cap = crate::user_install::open_user_root_for_setup(temp.path()).unwrap();
            let backup = Path::new("bin").join(&record.backup_name);
            let (path, bytes) = if record_removal {
                fs::remove_file(temp.path().join(&backup)).unwrap();
                (
                    PathBuf::from(RECORD),
                    fs::read(temp.path().join(RECORD)).unwrap(),
                )
            } else {
                (backup.clone(), b"old image".to_vec())
            };
            crate::user_install::interrupt_setup_remove(&cap, &path, &bytes);
            finish_loaded(load(&executable, None).unwrap(), false).unwrap();
            assert!(!temp.path().join(backup).exists());
            assert!(!temp.path().join(RECORD).exists());
            assert!(
                crate::user_install::inspect_setup_write_claim(&cap, &path, MAX_BINARY_BYTES)
                    .unwrap()
                    .is_none()
            );
        }
    }
}
