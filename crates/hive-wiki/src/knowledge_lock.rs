//! Kernel-backed exclusion with conservative compatibility for predecessor marker locks.
use super::{
    capability_parent, capability_temp_name, WikiError, LOCK_RELATIVE, LOCK_TIMEOUT,
    SHARED_OPERATION_LOCK_RELATIVE,
};
use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    time::{Duration, Instant},
};

const MARKER: &[u8] = b"schema_version=2\n";
const LEGACY: &[u8] = b"schema_version=1\n";

pub(super) struct CapabilityKnowledgeLock {
    parent: Dir,
    name: OsString,
    identity: (u64, u64),
    lease: File,
}

impl CapabilityKnowledgeLock {
    pub(super) fn acquire(root: &Dir) -> Result<Self, WikiError> {
        Self::acquire_at(root, Path::new(LOCK_RELATIVE), LOCK_TIMEOUT)
    }

    pub(super) fn acquire_shared_operation(root: &Dir) -> Result<Self, WikiError> {
        Self::acquire_at(
            root,
            Path::new(SHARED_OPERATION_LOCK_RELATIVE),
            LOCK_TIMEOUT,
        )
    }

    fn acquire_at(root: &Dir, relative: &Path, timeout: Duration) -> Result<Self, WikiError> {
        let started = Instant::now();
        loop {
            if let Some(lock) = Self::attempt(root, relative)? {
                return Ok(lock);
            }
            if started.elapsed() >= timeout {
                return Err(WikiError::Conflict("timed out waiting for canonical knowledge integration lock; a legacy marker is never assumed stale".into()));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn attempt(root: &Dir, relative: &Path) -> Result<Option<Self>, WikiError> {
        let (parent, name) = capability_parent(root, relative, true)?
            .ok_or_else(|| WikiError::Io("knowledge lock parent disappeared".into()))?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).follow(FollowSymlinks::No);
        let mut opened = match parent.open_with(&name, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Self::publish(parent, name)
            }
            Err(error) if busy(&error) => return Ok(None),
            Err(error) => return Err(failure(&error)),
        };
        let metadata = opened.metadata().map_err(|error| failure(&error))?;
        if !metadata.is_file() || metadata.len() > 64 {
            return Err(WikiError::Conflict(
                "unrecognized knowledge lock object".into(),
            ));
        }
        let mut bytes = Vec::new();
        match Read::by_ref(&mut opened).take(65).read_to_end(&mut bytes) {
            Ok(_) => {}
            Err(error) if busy(&error) => return Ok(None),
            Err(error) => return Err(failure(&error)),
        }
        if bytes == LEGACY || bytes.is_empty() {
            return Ok(None);
        }
        if bytes != MARKER {
            return Err(WikiError::Conflict(
                "unrecognized knowledge lock marker; preserved".into(),
            ));
        }
        let identity = (metadata.dev(), metadata.ino());
        let mut lease = opened.into_std();
        if !try_lock(&lease)? {
            return Ok(None);
        }
        if !current_identity(&parent, &name, identity)? {
            return Ok(None);
        }
        if !marker_matches(&mut lease)? {
            return Err(WikiError::Conflict(
                "knowledge lock marker changed; preserved".into(),
            ));
        }
        Ok(Some(Self {
            parent,
            name,
            identity,
            lease,
        }))
    }

    fn publish(parent: Dir, name: OsString) -> Result<Option<Self>, WikiError> {
        let temporary = capability_temp_name("hive-knowledge-lock");
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No);
        let opened = match parent.open_with(&temporary, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound || busy(&error) => {
                return Ok(None)
            }
            Err(error) => return Err(failure(&error)),
        };
        let metadata = opened.metadata().map_err(|error| failure(&error))?;
        let identity = (metadata.dev(), metadata.ino());
        let mut lease = opened.into_std();
        let prepared = (|| {
            if !try_lock(&lease)? {
                return Ok(false);
            }
            lease
                .write_all(MARKER)
                .and_then(|()| lease.sync_all())
                .map_err(|error| failure(&error))?;
            match parent.hard_link(&temporary, &parent, &name) {
                Ok(()) => Ok(true),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists || busy(&error) => {
                    Ok(false)
                }
                Err(error) => Err(failure(&error)),
            }
        })();
        let _ = parent.remove_file(&temporary);
        match prepared {
            Ok(true) => Ok(Some(Self {
                parent,
                name,
                identity,
                lease,
            })),
            Ok(false) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

fn current_identity(parent: &Dir, name: &OsStr, identity: (u64, u64)) -> Result<bool, WikiError> {
    match parent.symlink_metadata(name) {
        Ok(metadata) => Ok(metadata.is_file()
            && !metadata.file_type().is_symlink()
            && (metadata.dev(), metadata.ino()) == identity),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(failure(&error)),
    }
}

fn marker_matches(file: &mut File) -> Result<bool, WikiError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|error| failure(&error))?;
    let mut bytes = Vec::new();
    file.take(65)
        .read_to_end(&mut bytes)
        .map_err(|error| failure(&error))?;
    Ok(bytes == MARKER)
}

fn try_lock(file: &File) -> Result<bool, WikiError> {
    match file.try_lock() {
        Ok(()) => Ok(true),
        Err(std::fs::TryLockError::WouldBlock) => Ok(false),
        Err(std::fs::TryLockError::Error(error)) => Err(failure(&error)),
    }
}

fn busy(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::WouldBlock
        || (cfg!(windows) && matches!(error.raw_os_error(), Some(32 | 33)))
}

fn failure(error: &io::Error) -> WikiError {
    WikiError::Io(format!(
        "cannot acquire canonical knowledge integration lock: {error}"
    ))
}

impl Drop for CapabilityKnowledgeLock {
    fn drop(&mut self) {
        if current_identity(&self.parent, &self.name, self.identity).unwrap_or(false)
            && marker_matches(&mut self.lease).unwrap_or(false)
        {
            // Remove only while the lease is still held. Waiters verify the path identity again.
            let _ = self.parent.remove_file(&self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cap_std::ambient_authority;

    #[test]
    fn kernel_lock_excludes_another_writer_and_a_released_marker_can_be_reused() {
        let root = tempfile::tempdir().unwrap();
        let dir = Dir::open_ambient_dir(root.path(), ambient_authority()).unwrap();
        let held = CapabilityKnowledgeLock::acquire(&dir).unwrap();
        assert!(CapabilityKnowledgeLock::acquire_at(
            &dir,
            Path::new(LOCK_RELATIVE),
            Duration::ZERO
        )
        .is_err());
        drop(held);
        std::fs::write(root.path().join(LOCK_RELATIVE), MARKER).unwrap();
        drop(CapabilityKnowledgeLock::acquire(&dir).unwrap());
    }

    #[test]
    fn legacy_or_unknown_markers_are_never_reclaimed_without_proof() {
        let root = tempfile::tempdir().unwrap();
        let dir = Dir::open_ambient_dir(root.path(), ambient_authority()).unwrap();
        std::fs::create_dir_all(root.path().join(".hive/index")).unwrap();
        for bytes in [LEGACY, b"foreign lock data".as_slice()] {
            std::fs::write(root.path().join(LOCK_RELATIVE), bytes).unwrap();
            assert!(CapabilityKnowledgeLock::acquire_at(
                &dir,
                Path::new(LOCK_RELATIVE),
                Duration::ZERO
            )
            .is_err());
            assert_eq!(
                std::fs::read(root.path().join(LOCK_RELATIVE)).unwrap(),
                bytes
            );
        }
    }

    #[test]
    fn releasing_a_lock_preserves_a_replacement_at_its_path() {
        let root = tempfile::tempdir().unwrap();
        let dir = Dir::open_ambient_dir(root.path(), ambient_authority()).unwrap();
        let held = CapabilityKnowledgeLock::acquire(&dir).unwrap();
        let path = root.path().join(LOCK_RELATIVE);
        std::fs::rename(&path, root.path().join("retained-original.lock")).unwrap();
        std::fs::write(&path, b"foreign replacement").unwrap();
        drop(held);
        assert_eq!(std::fs::read(path).unwrap(), b"foreign replacement");
    }

    #[test]
    fn independent_contenders_never_enter_the_same_critical_section() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        let root = tempfile::tempdir().unwrap();
        let dir = Arc::new(Dir::open_ambient_dir(root.path(), ambient_authority()).unwrap());
        let active = Arc::new(AtomicUsize::new(0));
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let dir = Arc::clone(&dir);
                let active = Arc::clone(&active);
                std::thread::spawn(move || {
                    for _ in 0..20 {
                        let lock = CapabilityKnowledgeLock::acquire(&dir).unwrap();
                        assert_eq!(active.fetch_add(1, Ordering::SeqCst), 0);
                        std::thread::yield_now();
                        assert_eq!(active.fetch_sub(1, Ordering::SeqCst), 1);
                        drop(lock);
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }
}
