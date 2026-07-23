//! Provider-neutral invariants shared by Aigent Hive commands.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

/// Marker that distinguishes the Hive source workspace from a consumer project.
pub const SOURCE_MARKER_FILE: &str = "hive-source.json";

/// Errors raised before a command is allowed to mutate a target project.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TargetGuardError {
    /// The target is the Hive source workspace, where consumer artifacts are forbidden.
    SourceWorkspace { marker: PathBuf },
}

impl Display for TargetGuardError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceWorkspace { marker } => write!(
                formatter,
                "consumer setup is forbidden in the Hive source workspace: {}",
                marker.display()
            ),
        }
    }
}

impl Error for TargetGuardError {}

/// Return the source marker path for a target directory.
#[must_use]
pub fn source_marker_path(target: &Path) -> PathBuf {
    target.join(SOURCE_MARKER_FILE)
}

/// Reject a target directory when it is the Hive source workspace.
///
/// This check must run before setup, render, migration, or update opens any
/// destination file for writing.
///
/// # Errors
///
/// Returns [`TargetGuardError::SourceWorkspace`] when the target contains the
/// Hive source marker.
pub fn ensure_consumer_target(target: &Path) -> Result<(), TargetGuardError> {
    let marker = source_marker_path(target);
    if marker.is_file() {
        return Err(TargetGuardError::SourceWorkspace { marker });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_consumer_target, source_marker_path, TargetGuardError};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("aigent-hive-{name}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn accepts_a_consumer_directory_without_the_source_marker() {
        let target = temporary_directory("consumer");
        fs::create_dir_all(&target).expect("temporary target should be created");

        let result = ensure_consumer_target(&target);

        fs::remove_dir_all(&target).expect("temporary target should be removed");
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_the_hive_source_workspace_before_mutation() {
        let target = temporary_directory("source");
        fs::create_dir_all(&target).expect("temporary target should be created");
        let marker = source_marker_path(&target);
        fs::write(&marker, b"{}").expect("source marker should be written");

        let result = ensure_consumer_target(&target);

        fs::remove_dir_all(&target).expect("temporary target should be removed");
        assert_eq!(result, Err(TargetGuardError::SourceWorkspace { marker }));
    }
}
