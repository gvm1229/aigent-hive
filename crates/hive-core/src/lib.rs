//! Provider-neutral invariants shared by Aigent Hive commands.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

pub mod role;
pub mod run;
pub mod usage_guard;

pub(crate) fn validate_json_schema(
    schema: &str,
    instance: &serde_json::Value,
    label: &str,
) -> Result<(), String> {
    let schema: serde_json::Value = serde_json::from_str(schema)
        .map_err(|error| format!("embedded {label} schema is invalid JSON: {error}"))?;
    jsonschema::meta::validate(&schema)
        .map_err(|error| format!("embedded {label} schema is invalid: {error}"))?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| format!("cannot compile embedded {label} schema: {error}"))?;
    validator
        .validate(instance)
        .map_err(|error| format!("{label} violates the JSON Schema contract: {error}"))
}

/// Marker that distinguishes the Hive source workspace from a consumer project.
pub const SOURCE_MARKER_FILE: &str = "hive-source.json";

/// Errors raised before a command is allowed to mutate a target project.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TargetGuardError {
    /// The target is the Hive source workspace, where consumer artifacts are forbidden.
    SourceWorkspace { marker: PathBuf },
    /// A target ancestor is a symbolic link, so writes could escape the project.
    SymlinkAncestor { path: PathBuf },
    /// A target path could not be inspected safely.
    PathInspectionFailed { path: PathBuf },
    /// A managed path is not a safe, project-relative lexical path.
    UnsafeRelativePath { path: PathBuf },
    /// A path addresses a namespace that Hive must never inspect or mutate.
    ForbiddenNamespace { path: PathBuf },
}

impl Display for TargetGuardError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceWorkspace { marker } => write!(
                formatter,
                "consumer setup is forbidden in the Hive source workspace: {}",
                marker.display()
            ),
            Self::SymlinkAncestor { path } => {
                write!(
                    formatter,
                    "target path contains a symbolic link: {}",
                    path.display()
                )
            }
            Self::PathInspectionFailed { path } => write!(
                formatter,
                "target path could not be inspected safely: {}",
                path.display()
            ),
            Self::UnsafeRelativePath { path } => {
                write!(
                    formatter,
                    "path must be a safe project-relative path: {}",
                    path.display()
                )
            }
            Self::ForbiddenNamespace { path } => {
                write!(
                    formatter,
                    "Hive must not access foreign namespace: {}",
                    path.display()
                )
            }
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
    ensure_target_ancestors_are_directories(target)?;
    let marker = source_marker_path(target);
    match fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.is_file() => {
            return Err(TargetGuardError::SourceWorkspace { marker });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) | Err(_) => {
            return Err(TargetGuardError::PathInspectionFailed { path: marker });
        }
    }
    Ok(())
}

fn ensure_target_ancestors_are_directories(target: &Path) -> Result<(), TargetGuardError> {
    if target
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(TargetGuardError::PathInspectionFailed {
            path: target.to_path_buf(),
        });
    }
    let absolute = if target.is_absolute() {
        target.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| TargetGuardError::PathInspectionFailed {
                path: target.to_path_buf(),
            })?
            .join(target)
    };
    let mut ancestors = absolute.ancestors().collect::<Vec<_>>();
    ancestors.reverse();
    for ancestor in ancestors {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(TargetGuardError::SymlinkAncestor {
                    path: ancestor.to_path_buf(),
                });
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(TargetGuardError::PathInspectionFailed {
                    path: ancestor.to_path_buf(),
                });
            }
            Ok(_) => {}
            Err(_) => {
                return Err(TargetGuardError::PathInspectionFailed {
                    path: ancestor.to_path_buf(),
                });
            }
        }
    }
    Ok(())
}

/// Validate a path without touching the filesystem.
///
/// Only non-empty relative paths made of normal components are accepted.
/// Foreign runtime and host configuration namespaces are rejected.
///
/// # Errors
///
/// Returns an error for absolute paths, traversal, platform prefixes, empty
/// paths, or forbidden first components.
pub fn validate_project_relative(path: &Path) -> Result<(), TargetGuardError> {
    let portable = validate_relative_lexical(path)?;
    if portable
        .split('/')
        .any(|part| matches!(part, ".omx" | ".omc" | ".codex" | ".claude" | ".agents"))
    {
        return Err(TargetGuardError::ForbiddenNamespace {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

/// Validate one exact Hive-managed host Skill projection path.
///
/// This is the only lexical validation surface that permits a path below a
/// host discovery namespace. It accepts only
/// `.agents/skills/<safe-name>/SKILL.md` or
/// `.claude/skills/<safe-name>/SKILL.md`.
///
/// # Errors
///
/// Returns an error for any unsafe lexical form or any path that is not one
/// exact host Skill projection file.
pub fn validate_hive_skill_projection_relative(path: &Path) -> Result<(), TargetGuardError> {
    let _ = validate_relative_lexical(path)?;
    if !is_hive_skill_projection_path(path) {
        return Err(TargetGuardError::ForbiddenNamespace {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn validate_relative_lexical(path: &Path) -> Result<String, TargetGuardError> {
    let Some(raw) = path.to_str() else {
        return Err(TargetGuardError::UnsafeRelativePath {
            path: path.to_path_buf(),
        });
    };
    #[cfg(windows)]
    let portable = raw.replace('\\', "/");
    #[cfg(not(windows))]
    let portable = raw.to_owned();
    #[cfg(windows)]
    let has_foreign_separator = false;
    #[cfg(not(windows))]
    let has_foreign_separator = raw.contains('\\');
    if portable.is_empty()
        || path.is_absolute()
        || has_foreign_separator
        || portable.contains(':')
        || portable.starts_with("//")
        || portable
            .split('/')
            .any(|part| part.is_empty() || part == "..")
    {
        return Err(TargetGuardError::UnsafeRelativePath {
            path: path.to_path_buf(),
        });
    }
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(TargetGuardError::UnsafeRelativePath {
            path: path.to_path_buf(),
        });
    }
    Ok(portable)
}

/// Return whether a path is one exact Hive-managed host Skill projection.
///
/// Host discovery namespaces remain foreign by default. The only accepted
/// exception is `<host-root>/skills/<safe-name>/SKILL.md` under `.agents` or
/// `.claude`; this function never accepts the directory itself or another file.
#[must_use]
pub fn is_hive_skill_projection_path(path: &Path) -> bool {
    let Some(raw) = path.to_str() else {
        return false;
    };
    #[cfg(windows)]
    let portable = raw.replace('\\', "/");
    #[cfg(not(windows))]
    let portable = raw.to_owned();
    #[cfg(not(windows))]
    if raw.contains('\\') {
        return false;
    }
    is_hive_skill_projection_portable(&portable)
}

fn is_hive_skill_projection_portable(path: &str) -> bool {
    let parts = path.split('/').collect::<Vec<_>>();
    parts.len() == 4
        && matches!(parts[0], ".agents" | ".claude")
        && parts[1] == "skills"
        && valid_skill_projection_name(parts[2])
        && parts[3] == "SKILL.md"
}

fn valid_skill_projection_name(name: &str) -> bool {
    (2..=63).contains(&name.len())
        && name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// Reject a destination when an existing component below `target` is a symlink.
///
/// # Errors
///
/// Returns [`TargetGuardError::SymlinkAncestor`] for the first symlink found.
pub fn ensure_no_symlink_ancestors(target: &Path, relative: &Path) -> Result<(), TargetGuardError> {
    validate_project_relative(relative)?;
    ensure_no_symlink_ancestors_validated(target, relative)
}

/// Reject a projected Skill destination when an existing component below
/// `target` is a symlink.
///
/// # Errors
///
/// Returns a lexical validation error unless `relative` is one exact host
/// Skill projection path, or [`TargetGuardError::SymlinkAncestor`] for the
/// first symlink found.
pub fn ensure_no_symlink_ancestors_for_hive_skill_projection(
    target: &Path,
    relative: &Path,
) -> Result<(), TargetGuardError> {
    validate_hive_skill_projection_relative(relative)?;
    ensure_no_symlink_ancestors_validated(target, relative)
}

fn ensure_no_symlink_ancestors_validated(
    target: &Path,
    relative: &Path,
) -> Result<(), TargetGuardError> {
    match fs::symlink_metadata(target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(TargetGuardError::SymlinkAncestor {
                path: target.to_path_buf(),
            });
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(TargetGuardError::PathInspectionFailed {
                path: target.to_path_buf(),
            });
        }
        Ok(_) => {}
        Err(_) => {
            return Err(TargetGuardError::PathInspectionFailed {
                path: target.to_path_buf(),
            });
        }
    }
    let mut current = target.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            unreachable!("validated path contains only normal components");
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(TargetGuardError::SymlinkAncestor { path: current });
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(_) => {
                return Err(TargetGuardError::PathInspectionFailed { path: current });
            }
        }
    }
    Ok(())
}

/// Return a lowercase SHA-256 digest with the product prefix.
#[must_use]
pub fn sha256_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::ensure_no_symlink_ancestors;
    use super::{
        ensure_consumer_target, is_hive_skill_projection_path, sha256_digest, source_marker_path,
        validate_hive_skill_projection_relative, validate_project_relative, TargetGuardError,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        fs::canonicalize(std::env::temp_dir())
            .expect("temporary root should have a stable path")
            .join(format!("aigent-hive-{name}-{}-{nonce}", std::process::id()))
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

    #[test]
    fn rejects_traversal_absolute_and_foreign_paths() {
        for unsafe_path in [
            "../escape",
            "safe/../escape",
            r"..\escape",
            r"safe\..\escape",
            r"C:\escape",
            "C:escape",
            r"\\server\share",
            "//server/share",
            ".codex/hooks",
            ".claude/hooks",
            ".agents/hooks",
            "nested/.omx/state",
            "nested/.omc/state",
        ] {
            assert!(
                validate_project_relative(PathBuf::from(unsafe_path).as_path()).is_err(),
                "unsafe path was accepted: {unsafe_path}"
            );
        }
        assert!(matches!(
            validate_project_relative(PathBuf::from(".omx/state").as_path()),
            Err(TargetGuardError::ForbiddenNamespace { .. })
        ));
    }

    #[test]
    fn permits_only_exact_safe_host_skill_projection_files() {
        for path in [
            ".agents/skills/hive-simple-question/SKILL.md",
            ".agents/skills/local-inspect/SKILL.md",
            ".claude/skills/setup-harness/SKILL.md",
        ] {
            let path = PathBuf::from(path);
            assert!(is_hive_skill_projection_path(&path));
            assert!(
                validate_project_relative(&path).is_err(),
                "generic validation must keep host namespaces forbidden: {path:?}"
            );
            assert!(validate_hive_skill_projection_relative(&path).is_ok());
        }

        for path in [
            ".agents",
            ".agents/skills",
            ".agents/skills/local-inspect",
            ".agents/skills/local-inspect/README.md",
            ".agents/skills/local-inspect/references/extra.md",
            ".agents/skills/../outside/SKILL.md",
            ".agents/skills/a/SKILL.md",
            ".agents/skills/Upper/SKILL.md",
            ".agents/skills/bad:name/SKILL.md",
            ".agents/skills/bad\\name/SKILL.md",
            ".claude/hooks/local-inspect/SKILL.md",
            ".codex/skills/local-inspect/SKILL.md",
            "nested/.agents/skills/local-inspect/SKILL.md",
        ] {
            let path = PathBuf::from(path);
            assert!(!is_hive_skill_projection_path(&path), "{path:?}");
            assert!(
                validate_project_relative(&path).is_err(),
                "near-miss host path was accepted: {path:?}"
            );
            assert!(
                validate_hive_skill_projection_relative(&path).is_err(),
                "near-miss projection path was accepted: {path:?}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn accepts_native_windows_separators_for_internal_relative_paths() {
        let relative = PathBuf::from(".hive").join("knowledge").join("Raw");
        assert!(validate_project_relative(&relative).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_destination_ancestor() {
        use std::os::unix::fs::symlink;

        let target = temporary_directory("symlink");
        let outside = temporary_directory("outside");
        fs::create_dir_all(&target).expect("target should be created");
        fs::create_dir_all(&outside).expect("outside should be created");
        symlink(&outside, target.join(".hive")).expect("symlink should be created");

        let result =
            ensure_no_symlink_ancestors(&target, PathBuf::from(".hive/config/a").as_path());

        fs::remove_dir_all(&target).expect("target should be removed");
        fs::remove_dir_all(&outside).expect("outside should be removed");
        assert!(matches!(
            result,
            Err(TargetGuardError::SymlinkAncestor { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_target_below_a_symlinked_parent() {
        use std::os::unix::fs::symlink;

        let temporary = temporary_directory("target-parent-symlink");
        fs::create_dir_all(&temporary).expect("temporary root should be created");
        let temporary =
            fs::canonicalize(&temporary).expect("temporary root should have a stable path");
        let actual = temporary.join("actual");
        let target = actual.join("consumer");
        let alias = temporary.join("alias");
        fs::create_dir_all(&target).expect("target should be created");
        symlink(&actual, &alias).expect("parent alias should be created");

        let result = ensure_consumer_target(&alias.join("consumer"));

        fs::remove_dir_all(&temporary).expect("temporary root should be removed");
        assert!(matches!(
            result,
            Err(TargetGuardError::SymlinkAncestor { .. })
        ));
    }

    #[test]
    fn digest_is_stable() {
        assert_eq!(
            sha256_digest(b"abc"),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
