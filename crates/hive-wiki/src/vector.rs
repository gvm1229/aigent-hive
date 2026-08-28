//! Capability-scoped files for optional vector state. Canonical knowledge is never mutated.

use crate::{
    capability_parent, open_capability_file_nofollow, read_capability_optional,
    CapabilityFileSnapshot, CapabilityFileState, PinnedRoot, WikiError,
};
use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use hive_core::{
    ensure_consumer_target, ensure_no_symlink_ancestors, normalize_platform_root, sha256_digest,
};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_DATABASE_BYTES: u64 = 512 * 1024 * 1024;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// One immutable database reference or the mutable working copy.
#[derive(Clone, Copy, Debug)]
pub enum DatabaseKind<'a> {
    Staging,
    Checkpoint(&'a str),
    Generation(&'a str),
}

/// Exact source-versus-user filesystem ownership boundary.
pub struct VectorFiles {
    root: PinnedRoot,
    source: bool,
}

/// An exclusive writer lease. Queries only read atomically published control files.
pub struct VectorWriterLease {
    _file: std::fs::File,
}

/// Validate a digest-derived path component, never an arbitrary caller path.
///
/// # Errors
/// Returns an error for a noncanonical SHA-256 component.
pub fn validate_id(value: &str) -> Result<(), WikiError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WikiError::InvalidInput(
            "vector path requires a lowercase SHA-256 ID".to_owned(),
        ));
    }
    Ok(())
}

impl VectorFiles {
    /// Open a source/user ownership boundary without creating files. The caller authorizes user setup.
    ///
    /// # Errors
    /// Rejects links, invalid source markers, and consumer/source boundary confusion.
    pub fn open(root: &Path, source: bool) -> Result<Self, WikiError> {
        let root = normalize_platform_root(&std::path::absolute(root).map_err(io_error)?);
        ensure_no_symlink_ancestors(&root, Path::new(hive_core::SOURCE_MARKER_FILE))
            .map_err(|error| WikiError::InvalidInput(error.to_string()))?;
        if source {
            crate::source::validate_root(&root)?;
        } else {
            ensure_consumer_target(&root)
                .map_err(|error| WikiError::InvalidInput(error.to_string()))?;
        }
        Ok(Self {
            root: PinnedRoot::open(&root)?,
            source,
        })
    }

    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root.canonical_path
    }

    #[must_use]
    pub fn data_relative(&self) -> &Path {
        Path::new(if self.source {
            ".agents/work/vector"
        } else {
            ".hive/index/vector"
        })
    }

    #[must_use]
    pub fn control_relative(&self) -> &Path {
        Path::new(if self.source {
            ".agents/work/vector-control"
        } else {
            ".hive/config/vector-state"
        })
    }

    /// Calculate a root-bound collection/visibility selector.
    ///
    /// # Errors
    /// Rejects unencodable scope metadata.
    pub fn scope_id(&self, key: &impl Serialize) -> Result<String, WikiError> {
        let path = self.root_path().to_string_lossy().replace('\\', "/");
        let path = if cfg!(windows) {
            path.to_lowercase()
        } else {
            path
        };
        let bytes =
            serde_json_canonicalizer::to_vec(&(self.source, path, key)).map_err(json_error)?;
        Ok(sha256_digest(&bytes)[7..].to_owned())
    }

    /// Resolve a generated runtime directory, without reading or writing it.
    ///
    /// # Errors
    /// Rejects an invalid runtime ID.
    pub fn runtime_path(&self, runtime: &str) -> Result<PathBuf, WikiError> {
        validate_id(runtime)?;
        self.worker_path(&self.data_relative().join("runtimes").join(runtime))
    }

    /// Resolve one generated database location.
    ///
    /// # Errors
    /// Rejects invalid scope or generation IDs.
    pub fn database_relative(
        &self,
        scope: &str,
        kind: DatabaseKind<'_>,
    ) -> Result<PathBuf, WikiError> {
        validate_id(scope)?;
        let base = self.data_relative().join("scopes").join(scope);
        Ok(match kind {
            DatabaseKind::Staging => base.join("staging.sqlite3"),
            DatabaseKind::Checkpoint(id) => {
                validate_id(id)?;
                base.join("checkpoints").join(id).join("index.sqlite3")
            }
            DatabaseKind::Generation(id) => {
                validate_id(id)?;
                base.join("generations").join(id).join("index.sqlite3")
            }
        })
    }

    /// Resolve the absolute name passed to the fixed worker, after ownership checks.
    ///
    /// # Errors
    /// Rejects invalid IDs or linked managed ancestors.
    pub fn database_path(&self, scope: &str, kind: DatabaseKind<'_>) -> Result<PathBuf, WikiError> {
        let relative = self.database_relative(scope, kind)?;
        if let Some((parent, name)) = capability_parent(&self.root.dir, &relative, false)? {
            match parent.symlink_metadata(&name) {
                Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
                    return Err(WikiError::Conflict(
                        "vector database path is not regular".to_owned(),
                    ));
                }
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                    return Err(io_error(error))
                }
                _ => (),
            }
        }
        self.worker_path(&relative)
    }

    fn control_path(&self, key: Option<&str>) -> Result<PathBuf, WikiError> {
        let name = if let Some(key) = key {
            validate_id(key)?;
            format!("scope-{key}.json")
        } else {
            "runtime.json".to_owned()
        };
        Ok(self.control_relative().join(name))
    }

    /// Read control authority separately from untrusted `SQLite` cache bytes.
    ///
    /// # Errors
    /// Rejects linked, oversized, malformed, or unknown control fields through the caller's type.
    pub fn read_control<T: DeserializeOwned>(
        &self,
        key: Option<&str>,
    ) -> Result<(Option<T>, Option<String>), WikiError> {
        let relative = self.control_path(key)?;
        let Some(bytes) = read_capability_optional(&self.root.dir, &relative)? else {
            return Ok((None, None));
        };
        Ok((
            Some(serde_json::from_slice(&bytes).map_err(json_error)?),
            Some(sha256_digest(&bytes)),
        ))
    }

    /// Compare-and-swap one small control record through the existing rollback-safe publisher.
    ///
    /// # Errors
    /// Rejects concurrent changes or failed atomic installation, preserving prior bytes.
    pub fn write_control<T: Serialize>(
        &self,
        key: Option<&str>,
        expected: Option<&str>,
        value: &T,
    ) -> Result<(), WikiError> {
        let relative = self.control_path(key)?;
        let actual =
            read_capability_optional(&self.root.dir, &relative)?.map(|bytes| sha256_digest(&bytes));
        if actual.as_deref() != expected {
            return Err(WikiError::Conflict(
                "vector control changed after planning".to_owned(),
            ));
        }
        let bytes = serde_json_canonicalizer::to_vec(value).map_err(json_error)?;
        if bytes.len() > 1024 * 1024 {
            return Err(WikiError::InvalidInput(
                "vector control exceeds limits".to_owned(),
            ));
        }
        let mut snapshots = [CapabilityFileSnapshot::capture(&self.root.dir, &relative)?];
        let captured = match &snapshots[0].current {
            CapabilityFileState::Missing => None,
            CapabilityFileState::File { bytes, .. } => Some(sha256_digest(bytes)),
        };
        if captured.as_deref() != expected {
            return Err(WikiError::Conflict(
                "vector control changed during capture".to_owned(),
            ));
        }
        crate::transactional_capability(&self.root.dir, &mut snapshots, |snapshots| {
            snapshots[0].install_staged(&self.root.dir, &bytes)?;
            Ok(())
        })
    }

    /// Commit runtime and scope approval together using the rollback-safe publisher.
    ///
    /// # Errors
    /// Rejects stale authority or failed installation without retaining a partial control change.
    pub fn write_control_pair<T: Serialize, U: Serialize>(
        &self,
        runtime: (Option<&str>, &T),
        scope: (&str, Option<&str>, &U),
    ) -> Result<(), WikiError> {
        let paths = [self.control_path(None)?, self.control_path(Some(scope.0))?];
        let mut snapshots = [
            CapabilityFileSnapshot::capture(&self.root.dir, &paths[0])?,
            CapabilityFileSnapshot::capture(&self.root.dir, &paths[1])?,
        ];
        for (snapshot, expected) in snapshots.iter().zip([runtime.0, scope.1]) {
            let digest = match &snapshot.current {
                CapabilityFileState::Missing => None,
                CapabilityFileState::File { bytes, .. } => Some(sha256_digest(bytes)),
            };
            if digest.as_deref() != expected {
                return Err(WikiError::Conflict(
                    "vector control pair changed".to_owned(),
                ));
            }
        }
        let bytes = [
            serde_json_canonicalizer::to_vec(runtime.1).map_err(json_error)?,
            serde_json_canonicalizer::to_vec(scope.2).map_err(json_error)?,
        ];
        if bytes.iter().any(|value| value.len() > 1024 * 1024) {
            return Err(WikiError::InvalidInput(
                "vector control exceeds limits".to_owned(),
            ));
        }
        crate::transactional_capability(&self.root.dir, &mut snapshots, |snapshots| {
            for (snapshot, bytes) in snapshots.iter_mut().zip(&bytes) {
                snapshot.install_staged(&self.root.dir, bytes)?;
            }
            Ok(())
        })
    }

    /// Acquire an immediate writer lease; busy scopes remain readable through their prior generation.
    ///
    /// # Errors
    /// Rejects linked locks and concurrent writers without waiting on model work.
    pub fn writer(&self, key: Option<&str>) -> Result<VectorWriterLease, WikiError> {
        let relative = self.control_path(key)?.with_extension("lock");
        let (parent, name) = self.parent(&relative, true)?;
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .follow(FollowSymlinks::No);
        let file = parent.open_with(&name, &options).map_err(io_error)?;
        if !file.metadata().map_err(io_error)?.is_file()
            || file.metadata().map_err(io_error)?.nlink() != 1
        {
            return Err(WikiError::Conflict(
                "vector writer lock is not a private regular file".to_owned(),
            ));
        }
        let file = file.into_std();
        file.try_lock()
            .map_err(|_| WikiError::Conflict("vector writer is busy".to_owned()))?;
        Ok(VectorWriterLease { _file: file })
    }

    /// Reserve a fresh runtime directory. It remains inactive until the control record points to it.
    ///
    /// # Errors
    /// Rejects linked ancestors and destination collisions.
    pub fn reserve_runtime(&self) -> Result<(String, PathBuf), WikiError> {
        let id = fresh_id();
        let relative = self.data_relative().join("runtimes").join(&id);
        let (parent, name) = self.parent(&relative, true)?;
        parent.create_dir(&name).map_err(io_error)?;
        Ok((id, self.worker_path(&relative)?))
    }

    /// Reserve a private per-operation directory for downloads and process temporary files.
    ///
    /// # Errors
    /// Rejects linked ancestors and collisions without replacing existing files.
    pub fn reserve_work(&self) -> Result<PathBuf, WikiError> {
        let relative = self.data_relative().join("work").join(fresh_id());
        let (parent, name) = self.parent(&relative, true)?;
        parent.create_dir(&name).map_err(io_error)?;
        self.worker_path(&relative)
    }

    /// Allocate a publication attempt ID, separate from the trusted content digest.
    #[must_use]
    pub fn fresh_snapshot_id() -> String {
        fresh_id()
    }

    /// Create only the managed staging parent, never a consumer project configuration.
    ///
    /// # Errors
    /// Rejects links or invalid scope IDs.
    pub fn prepare_staging(&self, scope: &str) -> Result<PathBuf, WikiError> {
        let relative = self.database_relative(scope, DatabaseKind::Staging)?;
        self.parent(&relative, true)?;
        self.worker_path(&relative)
    }

    /// Hash a closed, single-link database and reject all `SQLite` recovery sidecars.
    ///
    /// # Errors
    /// Rejects missing, linked, oversized, changing, or journal-bearing snapshots.
    pub fn database_digest(
        &self,
        scope: &str,
        kind: DatabaseKind<'_>,
    ) -> Result<String, WikiError> {
        let relative = self.database_relative(scope, kind)?;
        let (mut file, before) = self.open_database(&relative)?;
        let mut hasher = Sha256::new();
        let length = std::io::copy(
            &mut Read::by_ref(&mut file).take(MAX_DATABASE_BYTES + 1),
            &mut HashWriter(&mut hasher),
        )
        .map_err(io_error)?;
        let after = file.metadata().map_err(io_error)?;
        if length != before.len()
            || length > MAX_DATABASE_BYTES
            || before.modified().ok() != after.modified().ok()
            || before.len() != after.len()
        {
            return Err(WikiError::Conflict(
                "vector database changed while hashing".to_owned(),
            ));
        }
        Ok(finish_digest(hasher))
    }

    /// Retire a former control record's immutable copy while holding the scope writer lease.
    ///
    /// # Errors
    /// Rejects unexpected bytes, links, recovery sidecars, or failed removal.
    pub fn retire_database(
        &self,
        scope: &str,
        kind: DatabaseKind<'_>,
        expected: &str,
    ) -> Result<(), WikiError> {
        if matches!(kind, DatabaseKind::Staging) {
            return Err(WikiError::InvalidInput(
                "mutable state is not an obsolete snapshot".to_owned(),
            ));
        }
        let relative = self.database_relative(scope, kind)?;
        let Some((parent, name)) = capability_parent(&self.root.dir, &relative, false)? else {
            return Ok(());
        };
        match parent.symlink_metadata(&name) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(io_error(error)),
            Ok(_) => (),
        }
        if self.database_digest(scope, kind)? != expected {
            return Err(WikiError::Conflict(
                "obsolete vector snapshot changed; retained".to_owned(),
            ));
        }
        // The scope writer lease protects Hive writers. This removes only an owned cache name,
        // never a symlink target or bytes addressed through another hardlink outside this scope.
        // It does not promise inode-conditional unlink against an uncooperative same-UID writer.
        parent.remove_file(&name).map_err(io_error)?;
        if let Some(directory) = relative.parent() {
            if let Some((ancestor, leaf)) = capability_parent(&self.root.dir, directory, false)? {
                let _ = ancestor.remove_dir(leaf); // Empty generated directory only; never recurse.
            }
        }
        Ok(())
    }

    /// Remove a redundant mutable copy only while an exact immutable recovery copy still exists.
    /// The caller holds the scope writer lease. Missing staging is already clean.
    ///
    /// # Errors
    /// Rejects links, sidecars, drift, absent recovery bytes, and failed removal.
    pub fn discard_staging_copy(
        &self,
        scope: &str,
        recovery: DatabaseKind<'_>,
        expected: &str,
    ) -> Result<(), WikiError> {
        if matches!(recovery, DatabaseKind::Staging) {
            return Err(WikiError::InvalidInput(
                "staging cannot authenticate itself".to_owned(),
            ));
        }
        let relative = self.database_relative(scope, DatabaseKind::Staging)?;
        let Some((parent, name)) = capability_parent(&self.root.dir, &relative, false)? else {
            return Ok(());
        };
        for suffix in ["-journal", "-wal", "-shm"] {
            match parent
                .symlink_metadata(OsStr::new(&format!("{}{suffix}", name.to_string_lossy())))
            {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(io_error(error)),
                Ok(_) => {
                    return Err(WikiError::Verification(
                        "staging recovery sidecar must be retained".to_owned(),
                    ))
                }
            }
        }
        match parent.symlink_metadata(&name) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(io_error(error)),
            Ok(_) => (),
        }
        if self.database_digest(scope, recovery)? != expected
            || self.database_digest(scope, DatabaseKind::Staging)? != expected
        {
            return Err(WikiError::Conflict(
                "staging is not an exact redundant recovery copy".to_owned(),
            ));
        }
        parent.remove_file(&name).map_err(io_error)
    }

    /// Copy an authenticated snapshot without aliasing its inode or replacing an existing destination.
    ///
    /// # Errors
    /// Rejects corruption, links, concurrent replacement, and immutable destination conflicts.
    /// Failed or interrupted copies remain unpublished, at a never-reused attempt ID. The caller
    /// retries with `fresh_snapshot_id`; a failed mutable staging copy is quarantined first.
    pub fn copy_database(
        &self,
        scope: &str,
        source: DatabaseKind<'_>,
        destination: DatabaseKind<'_>,
        expected: &str,
    ) -> Result<(), WikiError> {
        let source = self.database_relative(scope, source)?;
        let destination = self.database_relative(scope, destination)?;
        let (mut input, before) = self.open_database(&source)?;
        let (parent, name) = self.parent(&destination, true)?;
        if parent.symlink_metadata(&name).is_ok() {
            return Err(WikiError::Conflict(
                "immutable vector destination exists".to_owned(),
            ));
        }
        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        // Exclusive creation has no link/unlink crash window and never replaces existing bytes.
        // Only the separate, verified control CAS makes this file visible to readers.
        let mut output = parent.open_with(&name, &options).map_err(io_error)?;
        let mut hasher = Sha256::new();
        let mut count = 0_u64;
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            let read = input.read(&mut buffer).map_err(io_error)?;
            if read == 0 {
                break;
            }
            count += read as u64;
            if count > MAX_DATABASE_BYTES {
                return Err(WikiError::Conflict("vector copy exceeds bounds".to_owned()));
            }
            hasher.update(&buffer[..read]);
            output.write_all(&buffer[..read]).map_err(io_error)?;
        }
        output.sync_all().map_err(io_error)?;
        if count != before.len() || finish_digest(hasher) != expected {
            return Err(WikiError::Verification(
                "vector copy differs from its trusted receipt".to_owned(),
            ));
        }
        let staged = output.metadata().map_err(io_error)?;
        let published = open_capability_file_nofollow(&parent, &name)
            .map_err(io_error)?
            .metadata()
            .map_err(io_error)?;
        if (staged.dev(), staged.ino()) != (published.dev(), published.ino()) {
            return Err(WikiError::Conflict(
                "vector copy identity changed".to_owned(),
            ));
        }
        if published.nlink() != 1 {
            return Err(WikiError::Conflict(
                "vector copy acquired another link".to_owned(),
            ));
        }
        Ok(())
    }

    /// Retain a failed mutable working copy and sidecars under new owned quarantine names.
    ///
    /// # Errors
    /// Rejects invalid scope IDs or failed local renames. Symlink targets are never followed.
    pub fn quarantine_staging(&self, scope: &str) -> Result<Vec<String>, WikiError> {
        let relative = self.database_relative(scope, DatabaseKind::Staging)?;
        let Some((parent, name)) = capability_parent(&self.root.dir, &relative, false)? else {
            return Ok(Vec::new());
        };
        let mut moved = Vec::new();
        for suffix in ["", "-journal", "-wal", "-shm"] {
            let original = OsString::from(format!("{}{suffix}", name.to_string_lossy()));
            if parent.symlink_metadata(&original).is_err() {
                continue;
            }
            let retained = OsString::from(format!("quarantine-{}{suffix}", fresh_id()));
            parent
                .rename(&original, &parent, &retained)
                .map_err(io_error)?;
            moved.push(
                relative
                    .parent()
                    .ok_or_else(|| WikiError::InvalidInput("vector parent is absent".to_owned()))?
                    .join(retained)
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
        Ok(moved)
    }

    // Windows must reopen and compare the normalized root; keep one fallible API on all hosts.
    #[cfg_attr(not(windows), allow(clippy::unnecessary_wraps))]
    fn worker_path(&self, relative: &Path) -> Result<PathBuf, WikiError> {
        // SQLite file URIs cannot interpret Windows verbatim device prefixes.
        // Retain the pinned capability for file operations; normalize only subprocess names.
        let path = normalize_platform_root(self.root_path());
        #[cfg(windows)]
        {
            use std::path::{Component, Prefix};
            let mut components = path.components();
            if let Some(Component::Prefix(prefix)) = components.next() {
                let normal = match prefix.kind() {
                    Prefix::VerbatimDisk(drive) => {
                        Some(PathBuf::from(format!("{}:\\", char::from(drive))))
                    }
                    Prefix::VerbatimUNC(server, share) => {
                        Some(PathBuf::from("\\\\").join(server).join(share))
                    }
                    _ => None,
                };
                if let Some(mut normal) = normal {
                    for component in components {
                        if component != Component::RootDir {
                            normal.push(component.as_os_str());
                        }
                    }
                    let reopened = PinnedRoot::open(&normal)?;
                    let expected = self.root.dir.dir_metadata().map_err(io_error)?;
                    let actual = reopened.dir.dir_metadata().map_err(io_error)?;
                    if (expected.dev(), expected.ino()) != (actual.dev(), actual.ino()) {
                        return Err(WikiError::Conflict(
                            "normalized vector root identifies a different directory".to_owned(),
                        ));
                    }
                    return Ok(normal.join(relative));
                }
            }
        }
        Ok(path.join(relative))
    }

    fn parent(&self, relative: &Path, create: bool) -> Result<(Dir, OsString), WikiError> {
        capability_parent(&self.root.dir, relative, create)?
            .ok_or_else(|| WikiError::Io("vector parent is absent".to_owned()))
    }

    fn open_database(
        &self,
        relative: &Path,
    ) -> Result<(cap_std::fs::File, cap_std::fs::Metadata), WikiError> {
        let (parent, name) = self.parent(relative, false)?;
        for suffix in ["-journal", "-wal", "-shm"] {
            if parent
                .symlink_metadata(OsStr::new(&format!("{}{suffix}", name.to_string_lossy())))
                .is_ok()
            {
                return Err(WikiError::Verification(
                    "vector snapshot has SQLite recovery sidecars".to_owned(),
                ));
            }
        }
        let declared = parent.symlink_metadata(&name).map_err(io_error)?;
        if !declared.is_file()
            || declared.file_type().is_symlink()
            || declared.len() > MAX_DATABASE_BYTES
        {
            return Err(WikiError::Verification(
                "vector snapshot is not a bounded regular file".to_owned(),
            ));
        }
        let file = open_capability_file_nofollow(&parent, &name).map_err(io_error)?;
        let metadata = file.metadata().map_err(io_error)?;
        if !metadata.is_file() || metadata.nlink() != 1 || metadata.len() > MAX_DATABASE_BYTES {
            return Err(WikiError::Verification(
                "vector snapshot is not a bounded single-link file".to_owned(),
            ));
        }
        if (declared.dev(), declared.ino()) != (metadata.dev(), metadata.ino()) {
            return Err(WikiError::Conflict(
                "vector snapshot identity changed while opening".to_owned(),
            ));
        }
        Ok((file, metadata))
    }
}

struct HashWriter<'a>(&'a mut Sha256);
impl Write for HashWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0.update(buffer);
        Ok(buffer.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn finish_digest(hasher: Sha256) -> String {
    use std::fmt::Write as _;
    let mut value = String::from("sha256:");
    for byte in hasher.finalize() {
        write!(&mut value, "{byte:02x}").expect("String formatting is infallible");
    }
    value
}

fn fresh_id() -> String {
    // Namespace uniqueness only; this is never an authorization token.
    let value = format!(
        "{}:{}:{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    sha256_digest(value.as_bytes())[7..].to_owned()
}

fn io_error(error: impl std::fmt::Display) -> WikiError {
    WikiError::Io(error.to_string())
}
fn json_error(error: impl std::fmt::Display) -> WikiError {
    WikiError::InvalidInput(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn temporary() -> tempfile::TempDir {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        fs::create_dir_all(&root).expect("work directory");
        tempfile::Builder::new()
            .prefix("vector-files-")
            .tempdir_in(root)
            .expect("temporary vector root")
    }

    #[test]
    fn paired_control_rejects_stale_scope_before_changing_runtime() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let scope = "e".repeat(64);
        files
            .write_control(None, None, &json!({"runtime":"old"}))
            .expect("old runtime");
        files
            .write_control(Some(&scope), None, &json!({"scope":"old"}))
            .expect("old scope");
        let (_, runtime_digest) = files
            .read_control::<serde_json::Value>(None)
            .expect("runtime");
        assert!(files
            .write_control_pair(
                (runtime_digest.as_deref(), &json!({"runtime":"new"})),
                (&scope, None, &json!({"scope":"new"}))
            )
            .is_err());
        assert_eq!(
            files
                .read_control::<serde_json::Value>(None)
                .expect("preserved")
                .0,
            Some(json!({"runtime":"old"}))
        );
        let (_, scope_digest) = files
            .read_control::<serde_json::Value>(Some(&scope))
            .expect("scope");
        files
            .write_control_pair(
                (runtime_digest.as_deref(), &json!({"runtime":"new"})),
                (&scope, scope_digest.as_deref(), &json!({"scope":"new"})),
            )
            .expect("paired update");
        assert_eq!(
            files
                .read_control::<serde_json::Value>(None)
                .expect("updated")
                .0,
            Some(json!({"runtime":"new"}))
        );
    }

    #[test]
    fn control_cas_and_writer_lease_preserve_prior_authority() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("user boundary");
        let lease = files.writer(None).expect("writer");
        assert!(files.writer(None).is_err());
        files
            .write_control(None, None, &json!({"enabled": false}))
            .expect("first control");
        let (value, digest) = files
            .read_control::<serde_json::Value>(None)
            .expect("control");
        assert_eq!(value, Some(json!({"enabled": false})));
        assert!(files
            .write_control(None, None, &json!({"enabled": true}))
            .is_err());
        files
            .write_control(None, digest.as_deref(), &json!({"enabled": true}))
            .expect("CAS");
        drop(lease);
        assert!(files.writer(None).is_ok());
    }

    #[test]
    fn immutable_copies_are_single_link_and_quarantined_staging_is_recoverable() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let scope = "a".repeat(64);
        let stage = files.prepare_staging(&scope).expect("stage");
        let bytes = b"synthetic closed database";
        fs::write(&stage, bytes).expect("database");
        let digest = files
            .database_digest(&scope, DatabaseKind::Staging)
            .expect("hash");
        assert_eq!(digest, sha256_digest(bytes));
        let id = &digest[7..];
        files
            .copy_database(
                &scope,
                DatabaseKind::Staging,
                DatabaseKind::Checkpoint(id),
                &digest,
            )
            .expect("checkpoint");
        files
            .copy_database(
                &scope,
                DatabaseKind::Staging,
                DatabaseKind::Generation(id),
                &digest,
            )
            .expect("generation");
        assert!(files
            .copy_database(
                &scope,
                DatabaseKind::Staging,
                DatabaseKind::Generation(id),
                &digest
            )
            .is_err());
        fs::write(&stage, b"interrupted change").expect("interruption");
        fs::write(format!("{}-journal", stage.display()), b"untrusted journal").expect("journal");
        assert!(files
            .database_digest(&scope, DatabaseKind::Staging)
            .is_err());
        let moved = files.quarantine_staging(&scope).expect("quarantine");
        assert_eq!(moved.len(), 2);
        files
            .copy_database(
                &scope,
                DatabaseKind::Checkpoint(id),
                DatabaseKind::Staging,
                &digest,
            )
            .expect("restore");
        assert_eq!(fs::read(stage).expect("restored"), bytes);
        assert_eq!(
            files
                .database_digest(&scope, DatabaseKind::Generation(id))
                .expect("generation unchanged"),
            digest
        );
    }

    #[test]
    fn obsolete_snapshot_cleanup_preserves_external_bytes_and_retries() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let scope = "a".repeat(64);
        let id = "b".repeat(64);
        let kind = DatabaseKind::Checkpoint(&id);
        let stage = files.prepare_staging(&scope).expect("stage");
        fs::write(&stage, b"owned checkpoint").expect("stage bytes");
        let digest = sha256_digest(b"owned checkpoint");
        files
            .copy_database(&scope, DatabaseKind::Staging, kind, &digest)
            .expect("copy");
        let path = files.database_path(&scope, kind).expect("path");
        let note = path.with_file_name("user-note.txt");
        fs::write(&note, b"unexpected addition").expect("note");
        assert!(files
            .retire_database(&scope, kind, &sha256_digest(b"other"))
            .is_err());
        assert!(path.exists());
        files
            .retire_database(&scope, kind, &digest)
            .expect("remove exact copy");
        files
            .retire_database(&scope, kind, &digest)
            .expect("resume after unlink");
        assert_eq!(
            fs::read(&note).expect("addition retained"),
            b"unexpected addition"
        );
        let external = root.path().join("external.sqlite3");
        fs::write(&external, b"external data").expect("external");
        fs::hard_link(&external, &path).expect("unexpected link");
        assert!(files.retire_database(&scope, kind, &digest).is_err());
        assert_eq!(
            fs::read(&external).expect("external retained"),
            b"external data"
        );
        assert!(files
            .retire_database(&scope, DatabaseKind::Staging, &digest)
            .is_err());
    }

    #[test]
    fn staging_cleanup_requires_a_verified_immutable_recovery_copy() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let scope = "a".repeat(64);
        let stage = files.prepare_staging(&scope).expect("stage");
        let digest = sha256_digest(b"recoverable bytes");
        fs::write(&stage, b"recoverable bytes").expect("stage bytes");
        let id = VectorFiles::fresh_snapshot_id();
        let kind = DatabaseKind::Generation(&id);
        assert!(files.discard_staging_copy(&scope, kind, &digest).is_err());
        files
            .copy_database(&scope, DatabaseKind::Staging, kind, &digest)
            .expect("recovery copy");
        let recovery = files.database_path(&scope, kind).expect("path");
        let note = stage.with_file_name("user-note.txt");
        fs::write(&note, b"preserve").expect("note");
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let held = fs::OpenOptions::new()
                .read(true)
                .share_mode(3)
                .open(&stage)
                .unwrap();
            assert!(files.discard_staging_copy(&scope, kind, &digest).is_err());
            assert!(stage.exists() && recovery.exists());
            drop(held);
        }
        files
            .discard_staging_copy(&scope, kind, &digest)
            .expect("cleanup");
        assert!(!stage.exists());
        assert_eq!(fs::read(&recovery).unwrap(), b"recoverable bytes");
        assert_eq!(fs::read(&note).unwrap(), b"preserve");
        files
            .discard_staging_copy(&scope, kind, &digest)
            .expect("idempotent retry");
        fs::write(&stage, b"changed staging").unwrap();
        assert!(files.discard_staging_copy(&scope, kind, &digest).is_err());
        assert_eq!(fs::read(&stage).unwrap(), b"changed staging");
        fs::remove_file(&stage).unwrap();
        fs::hard_link(&recovery, &stage).unwrap();
        assert!(files.discard_staging_copy(&scope, kind, &digest).is_err());
        assert!(stage.exists());
    }

    #[test]
    fn staging_cleanup_retains_orphan_journals_for_quarantine() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let scope = "a".repeat(64);
        let stage = files.prepare_staging(&scope).expect("stage");
        let journal = stage.with_file_name("staging.sqlite3-journal");
        fs::write(&journal, b"interrupted").unwrap();
        let id = VectorFiles::fresh_snapshot_id();
        assert!(files
            .discard_staging_copy(
                &scope,
                DatabaseKind::Generation(&id),
                &sha256_digest(b"none")
            )
            .is_err());
        assert_eq!(fs::read(&journal).unwrap(), b"interrupted");
        assert_eq!(
            files
                .quarantine_staging(&scope)
                .expect("retain journal")
                .len(),
            1
        );
    }

    #[test]
    fn linked_databases_and_escape_ids_are_rejected_without_foreign_mutation() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let scope = "b".repeat(64);
        let stage = files.prepare_staging(&scope).expect("stage");
        let foreign = root.path().join("foreign.sqlite3");
        fs::write(&foreign, b"foreign bytes").expect("foreign");
        fs::hard_link(&foreign, &stage).expect("hardlink");
        assert!(files
            .database_digest(&scope, DatabaseKind::Staging)
            .is_err());
        assert!(files
            .database_path("../../outside", DatabaseKind::Staging)
            .is_err());
        files
            .quarantine_staging(&scope)
            .expect("retain link without dereference");
        assert_eq!(
            fs::read(foreign).expect("foreign preserved"),
            b"foreign bytes"
        );
    }

    #[test]
    fn source_vectors_do_not_create_a_consumer_tree() {
        let root = temporary();
        fs::write(root.path().join("hive-source.json"), br#"{"schema_version":1,"kind":"aigent-hive-source-workspace","consumer_setup_allowed":false}"#).expect("source marker");
        for language in ["en", "ko"] {
            fs::create_dir_all(root.path().join("docs/facts").join(language)).expect("facts");
        }
        assert!(VectorFiles::open(root.path(), false).is_err());
        let files = VectorFiles::open(root.path(), true).expect("source files");
        let _lease = files.writer(None).expect("source writer");
        files.reserve_runtime().expect("source staging");
        assert!(!root.path().join(".hive").exists());
        assert!(root.path().join(".agents/work/vector").is_dir());
    }

    #[test]
    fn runtime_reservations_never_rename_or_overwrite_existing_directories() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let (id, destination) = files.reserve_runtime().expect("reserved");
        fs::write(destination.join("foreign"), b"preserved").expect("foreign");
        let (next_id, next_path) = files.reserve_runtime().expect("next reservation");
        assert_ne!(id, next_id);
        assert_ne!(destination, next_path);
        if cfg!(windows) {
            assert!(!destination.to_string_lossy().starts_with("\\\\?\\"));
            assert!(!files
                .database_path(&id, DatabaseKind::Staging)
                .expect("worker database path")
                .to_string_lossy()
                .starts_with("\\\\?\\"));
        }
        assert!(files
            .read_control::<serde_json::Value>(None)
            .expect("inactive")
            .0
            .is_none());
        assert_eq!(
            fs::read(destination.join("foreign")).expect("preserved"),
            b"preserved"
        );
    }

    #[test]
    fn failed_copy_is_unpublished_and_a_fresh_attempt_recovers() {
        let root = temporary();
        let files = VectorFiles::open(root.path(), false).expect("files");
        let scope = "d".repeat(64);
        let stage = files.prepare_staging(&scope).expect("stage");
        fs::write(stage, b"closed database").expect("database");
        let failed = VectorFiles::fresh_snapshot_id();
        assert!(files
            .copy_database(
                &scope,
                DatabaseKind::Staging,
                DatabaseKind::Checkpoint(&failed),
                &sha256_digest(b"wrong")
            )
            .is_err());
        assert!(files
            .read_control::<serde_json::Value>(Some(&scope))
            .expect("control")
            .0
            .is_none());
        let recovered = VectorFiles::fresh_snapshot_id();
        let digest = sha256_digest(b"closed database");
        files
            .copy_database(
                &scope,
                DatabaseKind::Staging,
                DatabaseKind::Checkpoint(&recovered),
                &digest,
            )
            .expect("fresh retry");
        assert_eq!(
            files
                .database_digest(&scope, DatabaseKind::Checkpoint(&recovered))
                .expect("single link"),
            digest
        );
        let path = files
            .database_path(&scope, DatabaseKind::Checkpoint(&failed))
            .expect("retained");
        assert!(path.is_file());
        assert!(!fs::read_dir(path.parent().expect("parent"))
            .expect("directory")
            .any(|entry| entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".vector-copy-")));
    }

    #[cfg(windows)]
    #[test]
    fn verbatim_root_alias_cannot_redirect_a_worker_to_another_directory() {
        let root = temporary();
        let base = root.path().canonicalize().expect("verbatim root");
        let exact = base.join("different ");
        let alias = base.join("different");
        fs::create_dir(&exact).expect("verbatim directory");
        fs::create_dir(&alias).expect("normal alias");
        let files = VectorFiles::open(&exact, false).expect("pin exact root");
        assert!(files.runtime_path(&"a".repeat(64)).is_err());
        assert!(!alias.join(".hive").exists());
        drop(files);
        fs::remove_dir(&exact).expect("remove exact empty fixture");
        fs::remove_dir(&alias).expect("remove alias empty fixture");
    }
}
