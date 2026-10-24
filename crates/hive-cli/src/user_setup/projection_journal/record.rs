//! Recover the record itself before replaying any of its project or user writes.
use super::{path, validate, Journal, SetupError, SetupProgress, LIMIT};
use crate::user_install::{
    inspect_setup_write_claim, lock_setup_state, read_user_setup_file, recover_user_setup_file,
    recover_user_setup_record, replace_user_setup_file,
};
use cap_std::fs::Dir;
use hive_core::sha256_digest;

fn checked(
    bytes: &[u8],
    progress: &SetupProgress,
    binding: Option<&str>,
) -> Result<(), SetupError> {
    let journal: Journal = serde_json::from_slice(bytes)
        .map_err(|_| SetupError::Verification("invalid retained projection record".into()))?;
    validate(&journal, progress)?;
    if binding.is_some_and(|expected| sha256_digest(bytes) != expected) {
        return Err(SetupError::Conflict(
            "retained projection record fingerprint changed".into(),
        ));
    }
    Ok(())
}

pub(super) fn read(
    root: &Dir,
    progress: &SetupProgress,
    binding: Option<&str>,
) -> Result<Option<Vec<u8>>, SetupError> {
    let relative = path(progress)?;
    if inspect_setup_write_claim(root, &relative, LIMIT)
        .map_err(SetupError::Conflict)?
        .is_none()
    {
        return read_user_setup_file(root, &relative, LIMIT).map_err(SetupError::Conflict);
    }
    let _lease =
        lock_setup_state(root, &progress.binding.target_id[7..]).map_err(SetupError::Conflict)?;
    let Some(claim) =
        inspect_setup_write_claim(root, &relative, LIMIT).map_err(SetupError::Conflict)?
    else {
        return read_user_setup_file(root, &relative, LIMIT).map_err(SetupError::Conflict);
    };
    let current = read_user_setup_file(root, &relative, LIMIT).map_err(SetupError::Conflict)?;
    for bytes in claim
        .before
        .iter()
        .chain(claim.staged.iter())
        .chain(current.iter())
    {
        checked(bytes, progress, binding)?;
    }
    if claim.before.is_some() && (claim.staged.is_some() || binding.is_none()) {
        return Err(SetupError::Conflict(
            "projection record removal lacks its original binding".into(),
        ));
    }
    let before = claim
        .before
        .as_deref()
        .map_or_else(|| "absent".into(), sha256_digest);
    let after = if claim.before.is_some() {
        "absent".into()
    } else {
        claim
            .staged
            .as_deref()
            .or(current.as_deref())
            .map_or_else(|| "absent".into(), sha256_digest)
    };
    if claim.before.is_some() {
        recover_user_setup_record(root, &relative, &before, &after, claim.before_mode)
            .map_err(SetupError::Conflict)?;
    } else {
        recover_user_setup_file(root, &relative, &before, &after, claim.before_mode)
            .map_err(SetupError::Conflict)?;
    }
    read_user_setup_file(root, &relative, LIMIT).map_err(SetupError::Conflict)
}

pub(super) fn create(root: &Dir, progress: &SetupProgress, bytes: &[u8]) -> Result<(), SetupError> {
    checked(bytes, progress, None)?;
    let _lease =
        lock_setup_state(root, &progress.binding.target_id[7..]).map_err(SetupError::Conflict)?;
    replace_user_setup_file(root, &path(progress)?, None, Some(bytes)).map_err(SetupError::Conflict)
}
