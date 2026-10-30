//! Preserve a cancelled monitor write without publishing its staged active configuration.
use super::{
    capability_parent, claim_name, inspect_setup_write_claim, read_optional_regular,
    recover_user_setup_file, recover_user_setup_record, replace_user_setup_file,
    setup_file_permissions, validate_retained_object, FilePermissions, InstallError,
    MAX_USER_FILE_BYTES,
};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use std::{ffi::OsStr, io, path::Path};

const CONFIG: &str = ".hive/runtime/usage-monitor/config.json";
const STATE: &str = ".hive/runtime/usage-monitor/state.json";

#[derive(Clone, Copy, Eq, PartialEq)]
enum Target {
    Config,
    State,
}
impl Target {
    fn path(self) -> &'static Path {
        Path::new(match self {
            Self::Config => CONFIG,
            Self::State => STATE,
        })
    }
    fn archive(self) -> &'static str {
        match self {
            Self::Config => "config-claim",
            Self::State => "state-claim",
        }
    }
}

fn fingerprint(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".into(), sha256_digest)
}

fn validate(
    claim: &Dir,
    before: &str,
    after: &str,
    mode: Option<u32>,
    target: Target,
) -> Result<(), InstallError> {
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
                (after != "absent").then(|| setup_file_permissions(target.path())),
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
            target.path(),
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
    retire(
        root,
        operation,
        before,
        after,
        mode,
        current_digest,
        intent,
        Target::Config,
    )
    .map_err(|e| e.message().into())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn retire_monitor_state_claim(
    root: &Dir,
    operation: &str,
    before: &str,
    after: &str,
    mode: Option<u32>,
    current_digest: &str,
    intent: &[u8],
) -> Result<(), String> {
    retire(
        root,
        operation,
        before,
        after,
        mode,
        current_digest,
        intent,
        Target::State,
    )
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
    target: Target,
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
    let path = target.path();
    let mut current = read_optional_regular(root, path, 65536)?;
    if target == Target::Config && current.is_none() {
        return Err(InstallError::Conflict(
            "cancelled monitor configuration is absent".into(),
        ));
    }
    if fingerprint(current.as_deref()) != current_digest {
        return Err(InstallError::Conflict(
            "monitor configuration changed before claim retirement".into(),
        ));
    }
    let (parent, _) = capability_parent(root, path, false)?
        .ok_or_else(|| InstallError::Conflict("monitor parent disappeared".into()))?;
    let name = claim_name(path);
    let mut original = match parent.open_dir_nofollow(&name) {
        Ok(claim) => {
            validate(&claim, before, after, mode, target)?;
            Some(claim)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(InstallError::Conflict(error.to_string())),
    };
    let base = Path::new(".hive/config/setup-progress/cancelled-monitor").join(operation);
    preserve_intent(root, &base.join("write.json"), intent)?;
    if target == Target::State && current.is_none() {
        if let Some(claim) = original.take() {
            for (name, archived) in [
                ("claimed.bin", "state-before.bin"),
                ("replacement.bin", "state-after.bin"),
            ] {
                if let Some(bytes) = read_optional_regular(&claim, Path::new(name), 65536)? {
                    preserve_intent(root, &base.join(archived), &bytes)?;
                }
            }
            drop(claim);
            recover_user_setup_record(root, path, before, after, mode)
                .map_err(InstallError::Conflict)?;
            current = read_optional_regular(root, path, 65536)?;
            if fingerprint(current.as_deref()) != before {
                return Err(InstallError::Conflict(
                    "cancelled monitor history restore changed; preserved".into(),
                ));
            }
            original = match parent.open_dir_nofollow(&name) {
                Ok(claim) => {
                    validate(&claim, before, after, mode, target)?;
                    Some(claim)
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => return Err(InstallError::Conflict(error.to_string())),
            };
        }
    }
    let archive = base.join(target.archive());
    let (destination, archive_name) = capability_parent(root, &archive, true)?
        .ok_or_else(|| InstallError::Conflict("monitor archive parent unavailable".into()))?;
    let retained = match destination.open_dir_nofollow(&archive_name) {
        Ok(claim) => {
            validate(&claim, before, after, mode, target)?;
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
        validate(&claim, before, after, mode, target)?;
        drop(claim);
        parent
            .rename(&name, &destination, &archive_name)
            .map_err(|e| InstallError::Conflict(e.to_string()))?;
        let claim = destination
            .open_dir_nofollow(&archive_name)
            .map_err(|e| InstallError::Conflict(e.to_string()))?;
        validate(&claim, before, after, mode, target)?;
    }
    if read_optional_regular(root, path, 65536)? != current {
        return Err(InstallError::Conflict(
            "monitor configuration changed during claim retirement; preserved".into(),
        ));
    }
    Ok(())
}
