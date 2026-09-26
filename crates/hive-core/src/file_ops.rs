//! File primitives shared by project activation and user installation.
//!
//! Callers own consent, pinned-root validation, expected-old-byte comparisons,
//! private staging directories, journaling and cleanup. These primitives do not
//! authorize an installation or promise atomicity across a collection of files.

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, File, OpenOptions};
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Component, Path};

/// Preserve the calling domain's diagnostic and cleanup boundary.
#[derive(Debug)]
pub enum StageError<E> {
    /// No new file was acquired; never delete the name after this error.
    Create(io::Error),
    /// A new file exists but its domain-owned permissions could not be applied.
    Configure(E),
    /// A new file exists but writing or synchronization failed.
    Write(io::Error),
}

fn leaf(name: &OsStr) -> io::Result<()> {
    let mut components = Path::new(name).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected one relative file name",
        ));
    }
    Ok(())
}

/// Stage new bytes without following a link or overwriting an existing file.
///
/// The caller retains any partially created file for its own recovery procedure.
/// Permissions are applied before writing; a success includes file synchronization.
///
/// # Errors
/// Reports creation, caller-owned configuration, or write/synchronization failure separately.
pub fn stage<E>(
    parent: &Dir,
    name: &OsStr,
    bytes: &[u8],
    configure: impl FnOnce(&File) -> Result<(), E>,
) -> Result<(), StageError<E>> {
    leaf(name).map_err(StageError::Create)?;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent
        .open_with(name, &options)
        .map_err(StageError::Create)?;
    configure(&file).map_err(StageError::Configure)?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
        .map_err(StageError::Write)
}

/// Publish a private staged or claimed object without replacing a concurrent writer.
///
/// The calling transaction must have already verified the staged bytes/permissions
/// and claimed any expected destination. The source remains available for recovery.
/// This same operation restores a claimed old object when the destination is absent.
///
/// # Errors
/// Rejects non-leaf names, occupied destinations and unsupported filesystem links.
pub fn publish_exclusive(
    source: &Dir,
    source_name: &OsStr,
    destination: &Dir,
    destination_name: &OsStr,
) -> io::Result<()> {
    leaf(source_name)?;
    leaf(destination_name)?;
    source.hard_link(source_name, destination, destination_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_and_restore_preserve_concurrent_and_prior_bytes() {
        let root = tempfile::tempdir().expect("root");
        let dir = Dir::open_ambient_dir(root.path(), cap_std::ambient_authority()).expect("pin");
        stage(&dir, OsStr::new("old"), b"prior", |_| Ok::<_, ()>(())).expect("stage");
        stage(&dir, OsStr::new("new"), b"desired", |_| Ok::<_, ()>(())).expect("stage");
        publish_exclusive(&dir, OsStr::new("new"), &dir, OsStr::new("live")).expect("publish");
        assert!(publish_exclusive(&dir, OsStr::new("old"), &dir, OsStr::new("live")).is_err());
        assert_eq!(dir.read("live").expect("live"), b"desired");
        assert_eq!(dir.read("old").expect("retained"), b"prior");
        assert!(matches!(
            stage(&dir, OsStr::new("live"), b"overwrite", |_| Ok::<_, ()>(())),
            Err(StageError::Create(_))
        ));
        dir.remove_file("live").expect("owned removal");
        publish_exclusive(&dir, OsStr::new("old"), &dir, OsStr::new("live")).expect("restore");
        assert_eq!(dir.read("live").expect("restored"), b"prior");
    }

    #[test]
    fn unsafe_names_and_permission_failure_do_not_publish_bytes() {
        let root = tempfile::tempdir().expect("root");
        let dir = Dir::open_ambient_dir(root.path(), cap_std::ambient_authority()).expect("pin");
        assert!(matches!(
            stage(&dir, OsStr::new("../outside"), b"bad", |_| Ok::<_, ()>(())),
            Err(StageError::Create(_))
        ));
        assert!(matches!(
            stage(&dir, OsStr::new("partial"), b"bad", |_| Err("permissions")),
            Err(StageError::Configure("permissions"))
        ));
        assert!(dir.read("partial").expect("retained empty file").is_empty());
    }
}
