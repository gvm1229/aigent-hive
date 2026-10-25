//! Preserve a cancelled monitor write without publishing its staged active configuration.
use super::{
    capability_parent, claim_name, inspect_setup_write_claim, read_optional_regular,
    recover_user_setup_file, replace_user_setup_file, setup_file_permissions,
    validate_retained_object, FilePermissions, InstallError, MAX_USER_FILE_BYTES,
};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use std::{ffi::OsStr, io, path::Path};

const CONFIG: &str = ".hive/runtime/usage-monitor/config.json";

fn validate(claim: &Dir, before: &str, after: &str, mode: Option<u32>) -> Result<(), InstallError> {
    for entry in claim
        .entries()
        .map_err(|e| InstallError::Conflict(e.to_string()))?
    {
        let entry = entry.map_err(|e| InstallError::Conflict(e.to_string()))?;
        let name = entry.file_name();
        let (digest, permissions) = if name == OsStr::new("claimed.bin") {
            (
                Some(before),
                Some(FilePermissions {
                    executable: !cfg!(unix) || mode.is_some_and(|value| value & 0o111 != 0),
                    unix_mode: mode,
                }),
            )
        } else if name == OsStr::new("replacement.bin") {
            (
                (after != "absent").then_some(after),
                (after != "absent").then(|| setup_file_permissions(Path::new(CONFIG))),
            )
        } else {
            return Err(InstallError::Conflict(
                "cancelled monitor claim contains an unknown object".into(),
            ));
        };
        validate_retained_object(
            claim,
            Path::new(&name),
            digest,
            permissions,
            Path::new(CONFIG),
            "cancelled monitor",
        )?;
    }
    Ok(())
}

fn preserve_intent(root: &Dir, path: &Path, bytes: &[u8]) -> Result<(), InstallError> {
    let current = read_optional_regular(root, path, MAX_USER_FILE_BYTES)?;
    if current.as_deref().is_some_and(|current| current != bytes) {
        return Err(InstallError::Conflict(
            "cancelled monitor archive changed; preserved".into(),
        ));
    }
    if inspect_setup_write_claim(root, path, MAX_USER_FILE_BYTES)
        .map_err(InstallError::Conflict)?
        .is_some()
    {
        recover_user_setup_file(root, path, "absent", &sha256_digest(bytes), None)
            .map_err(InstallError::Conflict)?;
    }
    if read_optional_regular(root, path, MAX_USER_FILE_BYTES)?.is_none() {
        replace_user_setup_file(root, path, None, Some(bytes)).map_err(InstallError::Conflict)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn retire_monitor_config_claim(
    root: &Dir,
    operation: &str,
    before: &str,
    after: &str,
    mode: Option<u32>,
    current_digest: &str,
    intent: &[u8],
) -> Result<(), String> {
    retire(root, operation, before, after, mode, current_digest, intent)
        .map_err(|e| e.message().into())
}

#[allow(clippy::too_many_arguments)]
fn retire(
    root: &Dir,
    operation: &str,
    before: &str,
    after: &str,
    mode: Option<u32>,
    current_digest: &str,
    intent: &[u8],
) -> Result<(), InstallError> {
    if operation.len() != 64
        || !operation
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(InstallError::Conflict(
            "invalid cancelled monitor operation".into(),
        ));
    }
    let path = Path::new(CONFIG);
    let current = read_optional_regular(root, path, 65536)?.ok_or_else(|| {
        InstallError::Conflict("cancelled monitor configuration is absent".into())
    })?;
    if sha256_digest(&current) != current_digest {
        return Err(InstallError::Conflict(
            "monitor configuration changed before claim retirement".into(),
        ));
    }
    let (parent, _) = capability_parent(root, path, false)?
        .ok_or_else(|| InstallError::Conflict("monitor parent disappeared".into()))?;
    let name = claim_name(path);
    let original = match parent.open_dir_nofollow(&name) {
        Ok(claim) => {
            validate(&claim, before, after, mode)?;
            Some(claim)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(InstallError::Conflict(error.to_string())),
    };
    let base = Path::new(".hive/config/setup-progress/cancelled-monitor").join(operation);
    preserve_intent(root, &base.join("write.json"), intent)?;
    let archive = base.join("config-claim");
    let (destination, target) = capability_parent(root, &archive, true)?
        .ok_or_else(|| InstallError::Conflict("monitor archive parent unavailable".into()))?;
    let retained = match destination.open_dir_nofollow(&target) {
        Ok(claim) => {
            validate(&claim, before, after, mode)?;
            true
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(InstallError::Conflict(error.to_string())),
    };
    if original.is_some() && retained {
        return Err(InstallError::Conflict(
            "both original and archived monitor claims exist; preserved".into(),
        ));
    }
    if let Some(claim) = original {
        validate(&claim, before, after, mode)?;
        drop(claim);
        parent
            .rename(&name, &destination, &target)
            .map_err(|e| InstallError::Conflict(e.to_string()))?;
        let claim = destination
            .open_dir_nofollow(&target)
            .map_err(|e| InstallError::Conflict(e.to_string()))?;
        validate(&claim, before, after, mode)?;
    }
    if read_optional_regular(root, path, 65536)?.as_deref() != Some(current.as_slice()) {
        return Err(InstallError::Conflict(
            "monitor configuration changed during claim retirement; preserved".into(),
        ));
    }
    Ok(())
}
