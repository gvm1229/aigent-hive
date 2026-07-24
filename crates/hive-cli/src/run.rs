use super::{emit_action_result, ActionResult, Evidence};
use crate::usage::{
    qualify_and_dispatch_with_runner, AutomaticDispatchError, SensorError, SystemCommandRunner,
    UsageGuardEvidence, UsageObservation,
};
use cap_fs_ext::{
    DirExt, FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt,
};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use hive_core::role::RoleDocument;
use hive_core::run::{
    prepare_dispatch_brief, validate_transition, verify_owner_continuity, CapabilityResolution,
    DispatchBrief, DispatchContractError, OwnerBinding, OwnerContinuity, RunPlan, RunState,
    RunStatus, RunStatusDocument, SupportLevel,
};
use hive_core::usage_guard::UsageSnapshot;
use hive_core::{ensure_consumer_target, sha256_digest, validate_project_relative};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CHECKPOINT_REQUEST_SCHEMA: &str =
    include_str!("../../../schemas/run-checkpoint-request.schema.json");
const MAX_EXPLICIT_FILE_BYTES: usize = 1024 * 1024;
const MAX_EVIDENCE_FILE_BYTES: usize = 256 * 1024;
const MAX_TOTAL_EVIDENCE_BYTES: usize = 1024 * 1024;
const MAX_HARNESS_CONFIG_BYTES: usize = 64 * 1024;
const MAX_RUNTIME_RECORD_BYTES: usize = 64 * 1024;
const FRESH_CAPABILITY_MAX_AGE: Duration = Duration::from_mins(1);
const NEW_STATUS_BODY: &[u8] = b"# Run status\n";
const CHECKPOINT_USAGE: &str = "\
Record one optimistic durable run checkpoint.

USAGE:
    hive run checkpoint --target <dir> --request <request.json> --capabilities <fresh-json> --output json
";
const RESUME_USAGE: &str = "\
Read and validate one durable run without mutation or spawning.

USAGE:
    hive run resume --target <dir> --run <run-id> --capabilities <fresh-json> [--dispatch-intent manual|automatic] [--account-digest <sha256:...> --role <role-id> [--threshold <1..99>]] --output json
";
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) enum AdapterError {
    Input(String),
    Safety(String),
    Conflict(String),
    OwnerBlocked(String),
    Unsupported(String),
    OwnerUnsupported(String),
    Verification(String),
    Internal(String),
    Rollback(String),
}

impl AdapterError {
    pub(crate) const fn status(&self) -> &'static str {
        match self {
            Self::Input(_) | Self::Internal(_) | Self::Rollback(_) => "error",
            Self::Safety(_) | Self::OwnerBlocked(_) => "blocked",
            Self::Conflict(_) => "conflict",
            Self::Unsupported(_) | Self::OwnerUnsupported(_) => "unsupported",
            Self::Verification(_) => "verification-failed",
        }
    }

    pub(crate) const fn exit_code(&self) -> u8 {
        match self {
            Self::Input(_) => 2,
            Self::Safety(_) | Self::Conflict(_) | Self::OwnerBlocked(_) => 3,
            Self::Unsupported(_) | Self::OwnerUnsupported(_) => 4,
            Self::Verification(_) => 5,
            Self::Internal(_) | Self::Rollback(_) => 10,
        }
    }

    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::Input(_) => "hive.invalid-input",
            Self::Safety(_) => "hive.run-blocked",
            Self::Conflict(_) => "hive.run-conflict",
            Self::OwnerBlocked(_) => "hive.run-owner-drift",
            Self::Unsupported(_) => "hive.run-unsupported",
            Self::OwnerUnsupported(_) => "hive.run-owner-unsupported",
            Self::Verification(_) => "hive.run-verification-failed",
            Self::Internal(_) => "hive.internal-error",
            Self::Rollback(_) => "hive.run-rollback-failed",
        }
    }

    pub(crate) fn message(&self) -> &str {
        match self {
            Self::Input(message)
            | Self::Safety(message)
            | Self::Conflict(message)
            | Self::OwnerBlocked(message)
            | Self::Unsupported(message)
            | Self::OwnerUnsupported(message)
            | Self::Verification(message)
            | Self::Internal(message)
            | Self::Rollback(message) => message,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FileSnapshot {
    Missing,
    File(Vec<u8>),
}

impl FileSnapshot {
    pub(crate) fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Missing => None,
            Self::File(bytes) => Some(bytes),
        }
    }
}

pub(crate) struct PinnedTarget {
    requested: PathBuf,
    dir: Dir,
}

impl PinnedTarget {
    pub(crate) fn open(target: &Path) -> Result<Self, AdapterError> {
        ensure_consumer_target(target).map_err(|error| AdapterError::Safety(error.to_string()))?;
        let absolute = absolute_lexical(target)?;
        let dir = open_directory_nofollow_path(&absolute)?;
        let pinned = Self {
            requested: absolute,
            dir,
        };
        if pinned
            .read_optional(Path::new("hive-source.json"), 4096)?
            .is_some()
        {
            return Err(AdapterError::Safety(
                "consumer run commands are forbidden in the Hive source workspace".to_owned(),
            ));
        }
        Ok(pinned)
    }

    pub(crate) fn read_required(
        &self,
        relative: &Path,
        max_bytes: usize,
    ) -> Result<Vec<u8>, AdapterError> {
        self.read_optional(relative, max_bytes)?.ok_or_else(|| {
            AdapterError::Input(format!(
                "required artifact is missing: {}",
                relative.display()
            ))
        })
    }

    pub(crate) fn requested_path(&self) -> &Path {
        &self.requested
    }

    pub(crate) fn read_optional(
        &self,
        relative: &Path,
        max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, AdapterError> {
        let Some((parent, file_name)) = self.parent_for(relative)? else {
            return Ok(None);
        };
        match parent.symlink_metadata(&file_name) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(AdapterError::Safety(format!(
                "cannot inspect {}: {error}",
                relative.display()
            ))),
            Ok(metadata) if metadata.is_file() => {
                let file_len = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
                if file_len > max_bytes {
                    return Err(AdapterError::Input(format!(
                        "artifact exceeds {max_bytes} bytes: {}",
                        relative.display()
                    )));
                }
                let bytes = read_parent_file(&parent, &file_name, max_bytes).map_err(|error| {
                    AdapterError::Safety(format!(
                        "cannot read no-follow artifact {}: {error}",
                        relative.display()
                    ))
                })?;
                Ok(Some(bytes))
            }
            Ok(_) => Err(AdapterError::Safety(format!(
                "artifact is not a no-follow regular file: {}",
                relative.display()
            ))),
        }
    }

    pub(crate) fn snapshot(&self, relative: &Path) -> Result<FileSnapshot, AdapterError> {
        self.snapshot_bounded(relative, MAX_EXPLICIT_FILE_BYTES)
    }

    pub(crate) fn snapshot_bounded(
        &self,
        relative: &Path,
        max_bytes: usize,
    ) -> Result<FileSnapshot, AdapterError> {
        Ok(self
            .read_optional(relative, max_bytes)?
            .map_or(FileSnapshot::Missing, FileSnapshot::File))
    }

    pub(crate) fn publish(
        &self,
        relative: &Path,
        expected: &FileSnapshot,
        desired: &[u8],
    ) -> Result<bool, AdapterError> {
        self.verify_current()?;
        let current = self.snapshot(relative)?;
        if current.bytes() == Some(desired) {
            return Ok(false);
        }
        if current.bytes() != expected.bytes() {
            return Err(AdapterError::Conflict(format!(
                "artifact changed after optimistic read: {}",
                relative.display()
            )));
        }
        let (parent, file_name) = self.parent_for(relative)?.ok_or_else(|| {
            AdapterError::Safety(format!(
                "artifact parent is missing: {}",
                relative.display()
            ))
        })?;
        publish_parent_file(&parent, &file_name, expected, desired).map_err(|error| {
            let message = format!("cannot publish {} atomically: {error}", relative.display());
            if matches!(
                error.kind(),
                io::ErrorKind::AlreadyExists | io::ErrorKind::NotFound | io::ErrorKind::WouldBlock
            ) {
                AdapterError::Conflict(message)
            } else {
                AdapterError::Internal(message)
            }
        })?;
        Ok(true)
    }

    fn ensure_runtime_parent(&self, relative: &Path) -> Result<(), AdapterError> {
        validate_project_relative(relative)
            .map_err(|error| AdapterError::Safety(error.to_string()))?;
        if !relative.starts_with(".hive/runtime") {
            return Err(AdapterError::Safety(
                "runtime publication escaped .hive/runtime".to_owned(),
            ));
        }
        let parent = relative
            .parent()
            .ok_or_else(|| AdapterError::Safety("runtime path has no parent".to_owned()))?;
        let mut current = self
            .dir
            .try_clone()
            .map_err(|error| AdapterError::Internal(error.to_string()))?;
        let mut walked = PathBuf::new();
        for component in parent.components() {
            let name = component.as_os_str();
            walked.push(name);
            match current.symlink_metadata(name) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    return Err(AdapterError::Safety(format!(
                        "runtime ancestor is not a no-follow directory: {}",
                        walked.display()
                    )));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    current.create_dir(name).map_err(|create_error| {
                        AdapterError::Safety(format!(
                            "cannot create Hive runtime directory {}: {create_error}",
                            walked.display()
                        ))
                    })?;
                }
                Err(error) => {
                    return Err(AdapterError::Safety(format!(
                        "cannot inspect runtime ancestor {}: {error}",
                        walked.display()
                    )));
                }
            }
            current = current.open_dir_nofollow(name).map_err(|error| {
                AdapterError::Safety(format!(
                    "cannot open runtime ancestor no-follow {}: {error}",
                    walked.display()
                ))
            })?;
        }
        Ok(())
    }

    pub(crate) fn publish_runtime(
        &self,
        relative: &Path,
        expected: &FileSnapshot,
        desired: &[u8],
    ) -> Result<bool, AdapterError> {
        self.ensure_runtime_parent(relative)?;
        self.publish(relative, expected, desired)
    }

    pub(crate) fn restore(
        &self,
        relative: &Path,
        snapshot: &FileSnapshot,
        published: &[u8],
    ) -> Result<(), AdapterError> {
        self.verify_current()?;
        let current = self.snapshot(relative)?;
        if current.bytes() != Some(published) {
            return Err(AdapterError::Rollback(format!(
                "rollback refused to overwrite changed artifact: {}",
                relative.display()
            )));
        }
        match snapshot {
            FileSnapshot::Missing => {
                let (parent, file_name) = self.parent_for(relative)?.ok_or_else(|| {
                    AdapterError::Rollback(format!(
                        "rollback parent disappeared: {}",
                        relative.display()
                    ))
                })?;
                parent.remove_file(&file_name).map_err(|error| {
                    AdapterError::Rollback(format!(
                        "cannot remove created artifact {}: {error}",
                        relative.display()
                    ))
                })
            }
            FileSnapshot::File(bytes) => {
                self.publish(relative, &current, bytes)?;
                Ok(())
            }
        }
    }

    fn parent_for(&self, relative: &Path) -> Result<Option<(Dir, OsString)>, AdapterError> {
        validate_project_relative(relative)
            .map_err(|error| AdapterError::Safety(error.to_string()))?;
        let file_name = relative
            .file_name()
            .ok_or_else(|| AdapterError::Input("artifact path has no file name".to_owned()))?
            .to_os_string();
        let mut current = self
            .dir
            .try_clone()
            .map_err(|error| AdapterError::Internal(error.to_string()))?;
        let mut walked = PathBuf::new();
        if let Some(parent) = relative.parent() {
            for component in parent.components() {
                let component = component.as_os_str();
                walked.push(component);
                match current.symlink_metadata(component) {
                    Ok(metadata) if metadata.is_dir() => {}
                    Ok(_) => {
                        return Err(AdapterError::Safety(format!(
                            "artifact ancestor is not a no-follow directory: {}",
                            walked.display()
                        )));
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                    Err(error) => {
                        return Err(AdapterError::Safety(format!(
                            "cannot inspect artifact ancestor {}: {error}",
                            walked.display()
                        )));
                    }
                }
                current = current.open_dir_nofollow(component).map_err(|error| {
                    AdapterError::Safety(format!(
                        "cannot open artifact ancestor no-follow {}: {error}",
                        walked.display()
                    ))
                })?;
            }
        }
        Ok(Some((current, file_name)))
    }

    fn verify_current(&self) -> Result<(), AdapterError> {
        let current = open_directory_nofollow_path(&self.requested).map_err(|error| {
            AdapterError::Conflict(format!(
                "target no longer resolves safely: {}",
                error.message()
            ))
        })?;
        let pinned_metadata = self
            .dir
            .dir_metadata()
            .map_err(|error| AdapterError::Internal(error.to_string()))?;
        let current_metadata = current
            .dir_metadata()
            .map_err(|error| AdapterError::Internal(error.to_string()))?;
        if CapMetadataExt::dev(&pinned_metadata) != CapMetadataExt::dev(&current_metadata)
            || CapMetadataExt::ino(&pinned_metadata) != CapMetadataExt::ino(&current_metadata)
        {
            return Err(AdapterError::Conflict(
                "target changed after it was pinned".to_owned(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn read_parent_file(
    parent: &Dir,
    name: &OsStr,
    max_bytes: usize,
) -> io::Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    let mut file = parent.open_with(name, &options)?;
    read_open_file(&mut file, max_bytes)
}

fn read_open_file(file: &mut impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(u64::try_from(max_bytes).unwrap_or(u64::MAX) + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file exceeds bounded read",
        ));
    }
    Ok(bytes)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

impl FileIdentity {
    fn from_metadata(metadata: &cap_std::fs::Metadata) -> Self {
        Self {
            device: CapMetadataExt::dev(metadata),
            inode: CapMetadataExt::ino(metadata),
        }
    }

    fn matches(self, metadata: &cap_std::fs::Metadata) -> bool {
        self == Self::from_metadata(metadata)
    }
}

struct TemporaryFile {
    name: OsString,
    identity: FileIdentity,
}

fn create_temporary(parent: &Dir, prefix: &str, bytes: &[u8]) -> io::Result<TemporaryFile> {
    for _ in 0..128 {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let name = OsString::from(format!(
            "{prefix}-{}-{epoch:x}-{counter:x}",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        match parent.open_with(&name, &options) {
            Ok(mut file) => {
                if let Err(error) = file
                    .write_all(bytes)
                    .and_then(|()| file.flush())
                    .and_then(|()| file.sync_all())
                {
                    drop(file);
                    let _ = parent.remove_file(&name);
                    return Err(error);
                }
                let metadata = file.metadata()?;
                if !metadata.is_file() {
                    drop(file);
                    let _ = parent.remove_file(&name);
                    return Err(io::Error::other(
                        "exclusive temporary is not a regular file",
                    ));
                }
                return Ok(TemporaryFile {
                    name,
                    identity: FileIdentity::from_metadata(&metadata),
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "cannot allocate exclusive same-directory temporary file",
    ))
}

fn publish_parent_file(
    parent: &Dir,
    destination: &OsStr,
    expected: &FileSnapshot,
    desired: &[u8],
) -> io::Result<()> {
    publish_parent_file_with_hooks(
        parent,
        destination,
        expected,
        desired,
        |_, _| Ok(()),
        |_, _| Ok(()),
    )
}

#[cfg(test)]
fn publish_parent_file_with_hook(
    parent: &Dir,
    destination: &OsStr,
    expected: &FileSnapshot,
    desired: &[u8],
    after_claim: impl FnOnce(&Dir, &OsStr) -> io::Result<()>,
) -> io::Result<()> {
    publish_parent_file_with_hooks(
        parent,
        destination,
        expected,
        desired,
        after_claim,
        |_, _| Ok(()),
    )
}

#[allow(clippy::too_many_lines)]
fn publish_parent_file_with_hooks(
    parent: &Dir,
    destination: &OsStr,
    expected: &FileSnapshot,
    desired: &[u8],
    after_claim: impl FnOnce(&Dir, &OsStr) -> io::Result<()>,
    before_temporary_cleanup: impl FnOnce(&Dir, &OsStr) -> io::Result<()>,
) -> io::Result<()> {
    let temporary = create_temporary(parent, ".aigent-hive-run", desired)?;
    match expected {
        FileSnapshot::Missing => {
            if let Err(error) = parent.hard_link(&temporary.name, parent, destination) {
                let _ = parent.remove_file(&temporary.name);
                return Err(error);
            }
            if let Err(error) = before_temporary_cleanup(parent, &temporary.name) {
                return Err(rollback_new_publication(
                    parent,
                    destination,
                    &temporary,
                    &format!("temporary cleanup failed: {error}"),
                ));
            }
            if let Err(error) = parent.remove_file(&temporary.name) {
                return Err(rollback_new_publication(
                    parent,
                    destination,
                    &temporary,
                    &format!("temporary cleanup failed: {error}"),
                ));
            }
            Ok(())
        }
        FileSnapshot::File(expected_bytes) => {
            let (quarantine, quarantine_name) = create_quarantine(parent)?;
            let claimed_name = OsStr::new("claimed");
            if let Err(error) = parent.rename(destination, &quarantine, claimed_name) {
                let _ = parent.remove_file(&temporary.name);
                drop(quarantine);
                let _ = parent.remove_dir(&quarantine_name);
                return Err(error);
            }
            let moved = match read_parent_file(&quarantine, claimed_name, MAX_EXPLICIT_FILE_BYTES) {
                Ok(bytes) => bytes,
                Err(error) => {
                    let _ = parent.remove_file(&temporary.name);
                    return Err(restore_claim_error(
                        parent,
                        destination,
                        quarantine,
                        &quarantine_name,
                        &format!("cannot verify claimed bytes: {error}"),
                    ));
                }
            };
            if moved != *expected_bytes {
                let _ = parent.remove_file(&temporary.name);
                return Err(restore_claim_error(
                    parent,
                    destination,
                    quarantine,
                    &quarantine_name,
                    "claimed bytes changed during optimistic publication",
                ));
            }
            if let Err(error) = after_claim(parent, destination) {
                let _ = parent.remove_file(&temporary.name);
                return Err(restore_claim_error(
                    parent,
                    destination,
                    quarantine,
                    &quarantine_name,
                    &format!("publication hook failed after claim: {error}"),
                ));
            }
            if let Err(error) = parent.hard_link(&temporary.name, parent, destination) {
                let _ = parent.remove_file(&temporary.name);
                return Err(restore_claim_error(
                    parent,
                    destination,
                    quarantine,
                    &quarantine_name,
                    &format!("exclusive publication was blocked: {error}"),
                ));
            }
            if let Err(error) = before_temporary_cleanup(parent, &temporary.name) {
                return Err(rollback_claimed_publication(
                    parent,
                    destination,
                    desired,
                    quarantine,
                    &quarantine_name,
                    &format!("temporary cleanup failed: {error}"),
                ));
            }
            if let Err(error) = parent.remove_file(&temporary.name) {
                return Err(rollback_claimed_publication(
                    parent,
                    destination,
                    desired,
                    quarantine,
                    &quarantine_name,
                    &format!("temporary cleanup failed: {error}"),
                ));
            }
            if let Err(error) = quarantine.remove_file(claimed_name) {
                return Err(rollback_claimed_publication(
                    parent,
                    destination,
                    desired,
                    quarantine,
                    &quarantine_name,
                    &format!("claim cleanup failed: {error}"),
                ));
            }
            drop(quarantine);
            let _ = parent.remove_dir(&quarantine_name);
            Ok(())
        }
    }
}

fn rollback_new_publication(
    parent: &Dir,
    destination: &OsStr,
    temporary: &TemporaryFile,
    reason: &str,
) -> io::Error {
    let (quarantine, quarantine_name) = match create_quarantine(parent) {
        Ok(created) => created,
        Err(error) => {
            return io::Error::other(format!(
                "{reason}; canonical destination was preserved because rollback quarantine creation failed: {error}"
            ));
        }
    };
    let claimed_name = OsStr::new("claimed");
    if let Err(error) = parent.rename(destination, &quarantine, claimed_name) {
        drop(quarantine);
        let _ = parent.remove_dir(&quarantine_name);
        return io::Error::other(format!(
            "{reason}; canonical destination was preserved because exclusive rollback claim failed: {error}"
        ));
    }
    let claimed_metadata = quarantine.symlink_metadata(claimed_name);
    if claimed_metadata
        .as_ref()
        .is_ok_and(|metadata| metadata.is_file() && temporary.identity.matches(metadata))
    {
        let cleanup_file = quarantine.remove_file(claimed_name);
        drop(quarantine);
        let cleanup = cleanup_file.and_then(|()| parent.remove_dir(&quarantine_name));
        return cleanup.err().map_or_else(
            || {
                io::Error::other(format!(
                    "{reason}; exact published destination rolled back and recovery link {} retained",
                    temporary.name.to_string_lossy()
                ))
            },
            |cleanup| {
                io::Error::other(format!(
                    "{reason}; exact published destination rolled back and recovery link {} retained, but rollback quarantine cleanup failed: {cleanup}",
                    temporary.name.to_string_lossy()
                ))
            },
        );
    }
    let inspection = claimed_metadata.err().map_or_else(String::new, |error| {
        format!("; claim inspection failed: {error}")
    });
    let restored = restore_claim_error(parent, destination, quarantine, &quarantine_name, reason);
    io::Error::other(format!(
        "{reason}; racing destination was preserved{inspection}: {restored}"
    ))
}

fn create_quarantine(parent: &Dir) -> io::Result<(Dir, OsString)> {
    for _ in 0..128 {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let name = OsString::from(format!(
            ".aigent-hive-run-claim-{}-{epoch:x}-{counter:x}",
            std::process::id()
        ));
        match parent.create_dir(&name) {
            Ok(()) => match parent.open_dir_nofollow(&name) {
                Ok(directory) => return Ok((directory, name)),
                Err(error) => {
                    let _ = parent.remove_dir(&name);
                    return Err(error);
                }
            },
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "cannot allocate exclusive claim quarantine",
    ))
}

fn restore_claim_error(
    parent: &Dir,
    destination: &OsStr,
    quarantine: Dir,
    quarantine_name: &OsStr,
    reason: &str,
) -> io::Error {
    let claimed_name = OsStr::new("claimed");
    match quarantine.hard_link(claimed_name, parent, destination) {
        Ok(()) => {
            let cleanup_file = quarantine.remove_file(claimed_name);
            drop(quarantine);
            let cleanup = cleanup_file.and_then(|()| parent.remove_dir(quarantine_name));
            cleanup.err().map_or_else(
                || io::Error::new(io::ErrorKind::WouldBlock, reason),
                |cleanup| {
                    io::Error::other(format!(
                        "{reason}; prior bytes restored but claim cleanup failed: {cleanup}"
                    ))
                },
            )
        }
        Err(restore) => {
            drop(quarantine);
            io::Error::new(
                io::ErrorKind::WouldBlock,
                format!(
                    "{reason}; racing destination was preserved and prior bytes remain in private recovery: {restore}"
                ),
            )
        }
    }
}

fn rollback_claimed_publication(
    parent: &Dir,
    destination: &OsStr,
    desired: &[u8],
    quarantine: Dir,
    quarantine_name: &OsStr,
    reason: &str,
) -> io::Error {
    let current = read_parent_file(parent, destination, MAX_EXPLICIT_FILE_BYTES);
    if !matches!(current.as_deref(), Ok(bytes) if bytes == desired) {
        drop(quarantine);
        return io::Error::other(format!(
            "{reason}; live destination changed and was preserved while prior bytes remain recoverable"
        ));
    }
    if let Err(error) = parent.remove_file(destination) {
        drop(quarantine);
        return io::Error::other(format!(
            "{reason}; cannot remove exact published bytes before rollback: {error}"
        ));
    }
    let restored = restore_claim_error(parent, destination, quarantine, quarantine_name, reason);
    io::Error::other(format!("{reason}; publication rolled back: {restored}"))
}

pub(crate) fn read_explicit_file(path: &Path, max_bytes: usize) -> Result<Vec<u8>, AdapterError> {
    read_explicit_file_with_metadata(path, max_bytes).map(|(bytes, _)| bytes)
}

fn read_fresh_capability_file(path: &Path, max_bytes: usize) -> Result<Vec<u8>, AdapterError> {
    let (bytes, metadata) = read_explicit_file_with_metadata(path, max_bytes)?;
    let modified = metadata.modified().map_err(|error| {
        AdapterError::Input(format!(
            "cannot inspect fresh capability evidence timestamp: {error}"
        ))
    })?;
    let age = SystemTime::now()
        .duration_since(modified.into_std())
        .map_err(|_| {
            AdapterError::Input("fresh capability evidence timestamp is in the future".to_owned())
        })?;
    if age > FRESH_CAPABILITY_MAX_AGE {
        return Err(AdapterError::Input(
            "fresh capability evidence is older than 60 seconds".to_owned(),
        ));
    }
    Ok(bytes)
}

fn read_explicit_file_with_metadata(
    path: &Path,
    max_bytes: usize,
) -> Result<(Vec<u8>, cap_std::fs::Metadata), AdapterError> {
    read_explicit_file_with_metadata_and_hooks(path, max_bytes, |_, _| Ok(()), |_, _| Ok(()))
}

fn read_explicit_file_with_metadata_and_hooks(
    path: &Path,
    max_bytes: usize,
    after_preflight: impl FnOnce(&Dir, &OsStr) -> io::Result<()>,
    after_read: impl FnOnce(&Dir, &OsStr) -> io::Result<()>,
) -> Result<(Vec<u8>, cap_std::fs::Metadata), AdapterError> {
    let absolute = absolute_lexical(path)?;
    let parent_path = absolute
        .parent()
        .ok_or_else(|| AdapterError::Input("explicit input has no parent".to_owned()))?;
    let name = absolute
        .file_name()
        .ok_or_else(|| AdapterError::Input("explicit input has no file name".to_owned()))?;
    let parent = open_directory_nofollow_path(parent_path)?;
    let path_metadata = parent.symlink_metadata(name).map_err(|error| {
        AdapterError::Input(format!(
            "cannot inspect explicit input {}: {error}",
            path.display()
        ))
    })?;
    if !path_metadata.is_file()
        || path_metadata.len() > u64::try_from(max_bytes).unwrap_or(u64::MAX)
    {
        return Err(AdapterError::Input(format!(
            "explicit input must be a regular file no larger than {max_bytes} bytes"
        )));
    }
    let path_identity = FileIdentity::from_metadata(&path_metadata);
    after_preflight(&parent, name).map_err(|error| {
        AdapterError::Input(format!("explicit input pre-open boundary failed: {error}"))
    })?;
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    options.nonblock(true);
    let mut file = parent.open_with(name, &options).map_err(|error| {
        AdapterError::Input(format!("cannot open explicit input no-follow: {error}"))
    })?;
    let metadata = file.metadata().map_err(|error| {
        AdapterError::Input(format!("cannot inspect opened explicit input: {error}"))
    })?;
    if !metadata.is_file() || metadata.len() > u64::try_from(max_bytes).unwrap_or(u64::MAX) {
        return Err(AdapterError::Input(format!(
            "explicit input must be a regular file no larger than {max_bytes} bytes"
        )));
    }
    if !path_identity.matches(&metadata) {
        return Err(AdapterError::Input(
            "explicit input changed between preflight and open".to_owned(),
        ));
    }
    let opened_identity = FileIdentity::from_metadata(&metadata);
    let opened_len = metadata.len();
    let opened_modified = metadata
        .modified()
        .map_err(|error| {
            AdapterError::Input(format!(
                "cannot inspect opened explicit input timestamp: {error}"
            ))
        })?
        .into_std();
    let bytes = read_open_file(&mut file, max_bytes)
        .map_err(|error| AdapterError::Input(format!("cannot read explicit input: {error}")))?;
    after_read(&parent, name).map_err(|error| {
        AdapterError::Input(format!("explicit input post-read boundary failed: {error}"))
    })?;
    let final_metadata = file.metadata().map_err(|error| {
        AdapterError::Input(format!("cannot re-inspect opened explicit input: {error}"))
    })?;
    let final_modified = final_metadata
        .modified()
        .map_err(|error| {
            AdapterError::Input(format!(
                "cannot re-inspect opened explicit input timestamp: {error}"
            ))
        })?
        .into_std();
    if !final_metadata.is_file()
        || !opened_identity.matches(&final_metadata)
        || final_metadata.len() != opened_len
        || final_modified != opened_modified
        || u64::try_from(bytes.len()).unwrap_or(u64::MAX) != opened_len
    {
        return Err(AdapterError::Input(
            "explicit input changed while it was read".to_owned(),
        ));
    }
    Ok((bytes, final_metadata))
}

fn absolute_lexical(path: &Path) -> Result<PathBuf, AdapterError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .map_err(|error| {
                AdapterError::Input(format!("cannot resolve current directory: {error}"))
            })
    }
}

pub(crate) fn open_directory_nofollow_path(path: &Path) -> Result<Dir, AdapterError> {
    let mut root = PathBuf::new();
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => root.push(prefix.as_os_str()),
            Component::RootDir => root.push(component.as_os_str()),
            Component::Normal(name) => names.push(name.to_os_string()),
            Component::CurDir | Component::ParentDir => {
                return Err(AdapterError::Safety(format!(
                    "directory path is not lexically safe: {}",
                    path.display()
                )));
            }
        }
    }
    if root.as_os_str().is_empty() {
        return Err(AdapterError::Safety(format!(
            "directory path is not absolute: {}",
            path.display()
        )));
    }
    let mut current = Dir::open_ambient_dir(&root, ambient_authority()).map_err(|error| {
        AdapterError::Safety(format!(
            "cannot open filesystem root {}: {error}",
            root.display()
        ))
    })?;
    let mut walked = root;
    for name in names {
        walked.push(&name);
        match current.symlink_metadata(&name) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                return Err(AdapterError::Safety(format!(
                    "directory component is not a no-follow directory: {}",
                    walked.display()
                )));
            }
            Err(error) => {
                return Err(AdapterError::Safety(format!(
                    "cannot inspect directory component {}: {error}",
                    walked.display()
                )));
            }
        }
        current = current.open_dir_nofollow(&name).map_err(|error| {
            AdapterError::Safety(format!(
                "cannot open directory component no-follow {}: {error}",
                walked.display()
            ))
        })?;
    }
    Ok(current)
}

pub(crate) fn read_json_request<T: DeserializeOwned>(
    path: &Path,
    schema: &str,
    label: &str,
) -> Result<(T, Vec<u8>), AdapterError> {
    let bytes = read_explicit_file(path, MAX_EXPLICIT_FILE_BYTES)?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| AdapterError::Input(format!("invalid {label} JSON: {error}")))?;
    validate_schema(schema, &value, label)?;
    let typed = serde_json::from_value(value)
        .map_err(|error| AdapterError::Input(format!("invalid {label}: {error}")))?;
    Ok((typed, bytes))
}

fn validate_schema(schema: &str, value: &Value, label: &str) -> Result<(), AdapterError> {
    let schema: Value = serde_json::from_str(schema)
        .map_err(|error| AdapterError::Internal(format!("invalid embedded schema: {error}")))?;
    jsonschema::meta::validate(&schema)
        .map_err(|error| AdapterError::Internal(format!("invalid embedded schema: {error}")))?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .map_err(|error| AdapterError::Internal(format!("cannot compile schema: {error}")))?;
    validator
        .validate(value)
        .map_err(|error| AdapterError::Input(format!("{label} violates schema: {error}")))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointRequest {
    schema_version: u32,
    run_id: String,
    expected_revision: u64,
    state: RunState,
    passed_criteria: Vec<String>,
    failed_criteria: Vec<String>,
    active_roles: Vec<String>,
    next_action: Option<String>,
    #[serde(default)]
    latest_evidence: Vec<String>,
    blocker: Option<String>,
    resume_note: Option<String>,
    criterion_evidence: BTreeMap<String, Vec<String>>,
    updated_at: String,
}

struct CheckpointArguments {
    target: PathBuf,
    request: PathBuf,
    capabilities: PathBuf,
}

struct ResumeArguments {
    target: PathBuf,
    run_id: String,
    capabilities: PathBuf,
    dispatch_intent: DispatchIntent,
    account_digest: Option<String>,
    role_id: Option<String>,
    threshold: Option<u8>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DispatchIntent {
    Manual,
    Automatic,
}

impl DispatchIntent {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Automatic => "automatic",
        }
    }
}

pub(crate) fn run_run(arguments: &[String]) -> ExitCode {
    if arguments == ["checkpoint", "--help"] {
        print!("{CHECKPOINT_USAGE}");
        return ExitCode::SUCCESS;
    }
    if arguments == ["resume", "--help"] {
        print!("{RESUME_USAGE}");
        return ExitCode::SUCCESS;
    }
    let (action, result) = match arguments.first().map(String::as_str) {
        Some("checkpoint") => (
            "CheckpointRun",
            parse_checkpoint_arguments(&arguments[1..]).and_then(|parsed| checkpoint(&parsed)),
        ),
        Some("resume") => (
            "ResumeWork",
            parse_resume_arguments(&arguments[1..]).and_then(|parsed| resume(&parsed)),
        ),
        Some(other) => (
            "RunWork",
            Err(AdapterError::Input(format!("unknown run action: {other}"))),
        ),
        None => (
            "RunWork",
            Err(AdapterError::Input("run requires an action".to_owned())),
        ),
    };
    let result = result.unwrap_or_else(|error| failure_result(action, &error));
    emit_action_result(&result)
}

fn parse_checkpoint_arguments(arguments: &[String]) -> Result<CheckpointArguments, AdapterError> {
    let options = parse_options(arguments, &["--target", "--request", "--capabilities"])?;
    Ok(CheckpointArguments {
        target: PathBuf::from(required(&options, "--target")?),
        request: PathBuf::from(required(&options, "--request")?),
        capabilities: PathBuf::from(required(&options, "--capabilities")?),
    })
}

fn parse_resume_arguments(arguments: &[String]) -> Result<ResumeArguments, AdapterError> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--run",
            "--capabilities",
            "--dispatch-intent",
            "--account-digest",
            "--role",
            "--threshold",
        ],
    )?;
    let dispatch_intent = match optional(&options, "--dispatch-intent").unwrap_or("manual") {
        "manual" => DispatchIntent::Manual,
        "automatic" => DispatchIntent::Automatic,
        other => {
            return Err(AdapterError::Input(format!(
                "unsupported dispatch intent: {other}"
            )));
        }
    };
    let account_digest = optional(&options, "--account-digest").map(str::to_owned);
    let role_id = optional(&options, "--role").map(str::to_owned);
    let threshold = optional(&options, "--threshold")
        .map(|value| {
            value
                .parse::<u8>()
                .ok()
                .filter(|value| (1..=99).contains(value))
                .ok_or_else(|| {
                    AdapterError::Input(
                        "usage threshold must be an integer from 1 to 99".to_owned(),
                    )
                })
        })
        .transpose()?;
    match dispatch_intent {
        DispatchIntent::Manual
            if account_digest.is_some() || role_id.is_some() || threshold.is_some() =>
        {
            return Err(AdapterError::Input(
                "--account-digest, --role, and --threshold require --dispatch-intent automatic"
                    .to_owned(),
            ));
        }
        DispatchIntent::Automatic => {
            let digest = account_digest.as_deref().ok_or_else(|| {
                AdapterError::Input(
                    "--dispatch-intent automatic requires --account-digest".to_owned(),
                )
            })?;
            if !is_sha256_digest(digest) {
                return Err(AdapterError::Input(
                    "account digest must be sha256 followed by 64 lowercase hex digits".to_owned(),
                ));
            }
            let role_id = role_id.as_deref().ok_or_else(|| {
                AdapterError::Input("--dispatch-intent automatic requires --role".to_owned())
            })?;
            validate_project_relative(Path::new(role_id)).map_err(|_| {
                AdapterError::Input("automatic role must be one safe role identifier".to_owned())
            })?;
            if role_id.contains('/') || role_id.contains('\\') {
                return Err(AdapterError::Input(
                    "automatic role must be one safe role identifier".to_owned(),
                ));
            }
        }
        DispatchIntent::Manual => {}
    }
    Ok(ResumeArguments {
        target: PathBuf::from(required(&options, "--target")?),
        run_id: required(&options, "--run")?.to_owned(),
        capabilities: PathBuf::from(required(&options, "--capabilities")?),
        dispatch_intent,
        account_digest,
        role_id,
        threshold,
    })
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

pub(crate) fn parse_options<'a>(
    arguments: &'a [String],
    allowed: &[&str],
) -> Result<Vec<(&'a str, &'a str)>, AdapterError> {
    let mut options = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| AdapterError::Input(format!("missing value for {option}")))?;
        if option == "--output" {
            if value != "json" {
                return Err(AdapterError::Input(
                    "role and run commands require --output json".to_owned(),
                ));
            }
        } else if !allowed.contains(&option) {
            return Err(AdapterError::Input(format!("unknown option: {option}")));
        }
        if options.iter().any(|(existing, _)| *existing == option) {
            return Err(AdapterError::Input(format!("duplicate option: {option}")));
        }
        options.push((option, value.as_str()));
        index += 2;
    }
    if optional(&options, "--output") != Some("json") {
        return Err(AdapterError::Input(
            "role and run commands require --output json".to_owned(),
        ));
    }
    Ok(options)
}

pub(crate) fn required<'a>(
    options: &[(&'a str, &'a str)],
    name: &str,
) -> Result<&'a str, AdapterError> {
    optional(options, name)
        .ok_or_else(|| AdapterError::Input(format!("missing required option {name}")))
}

fn optional<'a>(options: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find_map(|(option, value)| (*option == name).then_some(*value))
}

fn failure_result(action: &'static str, error: &AdapterError) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message: error.message().to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

#[allow(clippy::too_many_lines)]
fn checkpoint(arguments: &CheckpointArguments) -> Result<ActionResult, AdapterError> {
    let (request, request_bytes) = read_json_request::<CheckpointRequest>(
        &arguments.request,
        CHECKPOINT_REQUEST_SCHEMA,
        "run checkpoint request",
    )?;
    let capability_bytes =
        read_fresh_capability_file(&arguments.capabilities, MAX_EXPLICIT_FILE_BYTES)?;
    let capability = CapabilityResolution::parse_json(&capability_bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    let target = PinnedTarget::open(&arguments.target)?;
    let plan_path = run_path(&request.run_id, "PLAN.md")?;
    let status_path = run_path(&request.run_id, "STATUS.md")?;
    let plan_bytes = target.read_required(&plan_path, MAX_EXPLICIT_FILE_BYTES)?;
    let plan = RunPlan::parse_markdown(&plan_bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    let roles = load_active_roles(&target, &request.active_roles, &request.run_id)?;
    let status_snapshot = target.snapshot(&status_path)?;
    let existing = match &status_snapshot {
        FileSnapshot::Missing => None,
        FileSnapshot::File(bytes) => Some(
            RunStatusDocument::parse_markdown(bytes)
                .map_err(|error| AdapterError::Verification(error.to_string()))?,
        ),
    };

    let (binding, body) = if let Some(existing) = existing.as_ref() {
        existing
            .validate_checkpoint()
            .map_err(|error| AdapterError::Verification(error.to_string()))?;
        if existing.status().run_id != request.run_id {
            return Err(AdapterError::Verification(
                "STATUS.md run_id does not match its path".to_owned(),
            ));
        }
        if as_set(plan.criteria()) != as_set(&existing.status().required_criteria) {
            return Err(AdapterError::Verification(
                "PLAN.md criteria differ from the existing checkpoint".to_owned(),
            ));
        }
        enforce_owner_continuity(existing, &capability)?;
        (
            existing.owner_binding().map_err(core_verification)?,
            existing.body().to_vec(),
        )
    } else {
        if request.expected_revision != 0 {
            return Err(AdapterError::Conflict(format!(
                "STATUS.md is missing at expected revision {}",
                request.expected_revision
            )));
        }
        (
            capability
                .owner_binding()
                .map_err(|error| AdapterError::Verification(error.to_string()))?,
            NEW_STATUS_BODY.to_vec(),
        )
    };

    let next_revision = request
        .expected_revision
        .checked_add(1)
        .ok_or_else(|| AdapterError::Input("checkpoint revision overflow".to_owned()))?;
    let desired = checkpoint_document(&request, plan.criteria(), &binding, next_revision, body)?;
    let desired_bytes = desired
        .encode_canonical()
        .map_err(|error| AdapterError::Verification(error.to_string()))?;

    if let Some(existing) = existing.as_ref() {
        let current_revision = existing.status().revision;
        if current_revision == next_revision {
            if status_snapshot.bytes() != Some(desired_bytes.as_slice()) {
                return Err(AdapterError::Conflict(format!(
                    "revision {current_revision} already contains different checkpoint bytes"
                )));
            }
        } else if current_revision == request.expected_revision {
            validate_transition(existing, &desired).map_err(core_verification)?;
        } else {
            return Err(AdapterError::Conflict(format!(
                "expected revision {}, found {current_revision}",
                request.expected_revision
            )));
        }
    }

    let verified_evidence = verify_checkpoint_evidence(&target, desired.status())?;
    let changed = target.publish(&status_path, &status_snapshot, &desired_bytes)?;
    let status_digest = sha256_digest(&desired_bytes);
    let mut evidence = vec![
        Evidence {
            kind: "file",
            locator: arguments.request.display().to_string(),
            digest: sha256_digest(&request_bytes),
        },
        Evidence {
            kind: "file",
            locator: arguments.capabilities.display().to_string(),
            digest: sha256_digest(&capability_bytes),
        },
        Evidence {
            kind: "file",
            locator: portable_relative_path(&plan_path),
            digest: sha256_digest(&plan_bytes),
        },
        Evidence {
            kind: "file",
            locator: portable_relative_path(&status_path),
            digest: status_digest,
        },
    ];
    evidence.extend(verified_evidence);
    evidence.extend(roles.iter().flat_map(LoadedRole::result_evidence));
    let data = serde_json::to_value(desired.status())
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    Ok(ActionResult {
        schema_version: 1,
        action: "CheckpointRun",
        status: "success",
        exit_code: 0,
        code: if changed {
            "hive.run-checkpointed"
        } else {
            "hive.run-checkpoint-idempotent"
        },
        message: if changed {
            "durable run checkpoint committed atomically".to_owned()
        } else {
            "identical durable run checkpoint already exists".to_owned()
        },
        changed_paths: changed
            .then(|| portable_relative_path(&status_path))
            .into_iter()
            .collect(),
        evidence,
        next_action: desired.status().next_action.clone(),
        data: Some(data),
    })
}

fn checkpoint_document(
    request: &CheckpointRequest,
    criteria: &[String],
    binding: &OwnerBinding,
    revision: u64,
    body: Vec<u8>,
) -> Result<RunStatusDocument, AdapterError> {
    if request.schema_version != 1 {
        return Err(AdapterError::Input(
            "unsupported checkpoint request version".to_owned(),
        ));
    }
    RunStatusDocument::from_status(
        RunStatus {
            schema_version: 1,
            run_id: request.run_id.clone(),
            revision,
            state: request.state,
            required_criteria: criteria.to_vec(),
            passed_criteria: request.passed_criteria.clone(),
            failed_criteria: request.failed_criteria.clone(),
            active_roles: request.active_roles.clone(),
            next_action: request.next_action.clone(),
            latest_evidence: request.latest_evidence.clone(),
            blocker: request.blocker.clone(),
            updated_at: request.updated_at.clone(),
            host: Some(binding.host),
            host_version: Some(binding.host_version.clone()),
            surface: Some(binding.surface),
            external_runtime: binding.external_runtime,
            resolved_owner: Some(binding.resolved_owner),
            resolution_evidence_digest: Some(binding.resolution_evidence_digest.clone()),
            subagent_support: Some(binding.subagent_support),
            resume_note: request.resume_note.clone(),
            criterion_evidence: request.criterion_evidence.clone(),
        },
        body,
    )
    .map_err(core_verification)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UsageHistoryRecord {
    schema_version: u32,
    snapshot: UsageSnapshot,
    evidence_digest: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DispatchAuthorizationRecord {
    schema_version: u32,
    authorization_id: String,
    run_id: String,
    status_revision: u64,
    role_id: String,
    brief_digest: String,
    usage_evidence_digest: String,
    state: String,
    record_digest: String,
}

fn installed_usage_threshold(target: &PinnedTarget) -> Result<u8, AdapterError> {
    let relative = Path::new(".hive/config/harness.toml");
    let bytes = target
        .read_optional(relative, MAX_HARNESS_CONFIG_BYTES)?
        .ok_or_else(|| {
            AdapterError::Safety(
                "automatic resume requires installed .hive/config/harness.toml".to_owned(),
            )
        })?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        AdapterError::Safety("installed harness.toml must be valid UTF-8".to_owned())
    })?;
    let mut threshold = None;
    let mut entered_table = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            entered_table = true;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(AdapterError::Safety(
                "installed harness.toml contains a malformed assignment".to_owned(),
            ));
        };
        if key.trim() != "usage_stop_remaining_percent" {
            continue;
        }
        if entered_table {
            return Err(AdapterError::Safety(
                "installed usage threshold must be a root harness.toml key".to_owned(),
            ));
        }
        if threshold.is_some() {
            return Err(AdapterError::Safety(
                "installed harness.toml contains duplicate usage threshold".to_owned(),
            ));
        }
        let raw_value = value.trim();
        let parsed = raw_value
            .parse::<u8>()
            .ok()
            .filter(|parsed| (1..=99).contains(parsed) && parsed.to_string() == raw_value);
        threshold = Some(parsed.ok_or_else(|| {
            AdapterError::Safety(
                "installed harness.toml usage threshold must be an integer from 1 to 99".to_owned(),
            )
        })?);
    }
    threshold.ok_or_else(|| {
        AdapterError::Safety(
            "installed harness.toml is missing usage_stop_remaining_percent".to_owned(),
        )
    })
}

fn usage_history_path(account_digest: &str) -> PathBuf {
    let key = sha256_digest(account_digest.as_bytes());
    Path::new(".hive/runtime/usage-history").join(format!(
        "{}.json",
        key.strip_prefix("sha256:").unwrap_or(&key)
    ))
}

fn read_usage_history(
    target: &PinnedTarget,
    account_digest: &str,
) -> Result<(PathBuf, FileSnapshot, Vec<UsageSnapshot>, &'static str), AdapterError> {
    let path = usage_history_path(account_digest);
    let snapshot = target.snapshot(&path)?;
    let Some(bytes) = snapshot.bytes() else {
        return Ok((path, snapshot, Vec::new(), "absent"));
    };
    if bytes.len() > MAX_RUNTIME_RECORD_BYTES {
        return Err(AdapterError::Safety(
            "usage history exceeds the bounded runtime record size".to_owned(),
        ));
    }
    let record: UsageHistoryRecord = serde_json::from_slice(bytes)
        .map_err(|_| AdapterError::Safety("usage history is malformed".to_owned()))?;
    if record.schema_version != 1
        || record.snapshot.account_scope_digest != account_digest
        || record.evidence_digest
            != sha256_digest(
                &serde_json_canonicalizer::to_vec(&record.snapshot)
                    .map_err(|error| AdapterError::Internal(error.to_string()))?,
            )
    {
        return Err(AdapterError::Safety(
            "usage history failed its account or integrity binding".to_owned(),
        ));
    }
    Ok((path, snapshot, vec![record.snapshot], "available"))
}

fn publish_usage_history(
    target: &PinnedTarget,
    path: &Path,
    expected: &FileSnapshot,
    snapshots: &[UsageSnapshot],
) -> Result<bool, AdapterError> {
    let [snapshot] = snapshots else {
        return Err(AdapterError::Internal(
            "normalized usage observation must contain one selected window".to_owned(),
        ));
    };
    let record = UsageHistoryRecord {
        schema_version: 1,
        snapshot: snapshot.clone(),
        evidence_digest: sha256_digest(
            &serde_json_canonicalizer::to_vec(snapshot)
                .map_err(|error| AdapterError::Internal(error.to_string()))?,
        ),
    };
    let bytes = serde_json_canonicalizer::to_vec(&record)
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    target.publish_runtime(path, expected, &bytes)
}

fn authorization_binding(
    run_id: &str,
    status_revision: u64,
    role_id: &str,
    brief: &DispatchBrief,
) -> Result<(String, String, PathBuf), AdapterError> {
    let brief_digest = sha256_digest(
        &serde_json_canonicalizer::to_vec(brief)
            .map_err(|error| AdapterError::Internal(error.to_string()))?,
    );
    let binding = json!({
        "run_id": run_id,
        "status_revision": status_revision,
        "role_id": role_id,
        "brief_digest": brief_digest,
    });
    let authorization_id = sha256_digest(
        &serde_json_canonicalizer::to_vec(&binding)
            .map_err(|error| AdapterError::Internal(error.to_string()))?,
    );
    let file_name = authorization_id
        .strip_prefix("sha256:")
        .unwrap_or(&authorization_id);
    let path = Path::new(".hive/runtime/dispatch-authorizations").join(format!("{file_name}.json"));
    Ok((authorization_id, brief_digest, path))
}

fn existing_authorization(
    target: &PinnedTarget,
    path: &Path,
    authorization_id: &str,
    run_id: &str,
    status_revision: u64,
    role_id: &str,
    brief_digest: &str,
) -> Result<Option<FileSnapshot>, AdapterError> {
    let snapshot = target.snapshot(path)?;
    let Some(bytes) = snapshot.bytes() else {
        return Ok(Some(snapshot));
    };
    if bytes.len() > MAX_RUNTIME_RECORD_BYTES {
        return Err(AdapterError::Safety(
            "dispatch authorization exceeds the bounded runtime record size".to_owned(),
        ));
    }
    let record: DispatchAuthorizationRecord = serde_json::from_slice(bytes)
        .map_err(|_| AdapterError::Safety("dispatch authorization is malformed".to_owned()))?;
    if record.schema_version != 1
        || record.authorization_id != authorization_id
        || record.run_id != run_id
        || record.status_revision != status_revision
        || record.role_id != role_id
        || record.brief_digest != brief_digest
        || record.state != "issued"
        || !is_sha256_digest(&record.usage_evidence_digest)
        || record.record_digest != authorization_record_digest(&record)?
    {
        return Err(AdapterError::Safety(
            "dispatch authorization failed its immutable binding".to_owned(),
        ));
    }
    Ok(None)
}

fn authorization_record_digest(
    record: &DispatchAuthorizationRecord,
) -> Result<String, AdapterError> {
    let payload = json!({
        "schema_version": record.schema_version,
        "authorization_id": record.authorization_id,
        "run_id": record.run_id,
        "status_revision": record.status_revision,
        "role_id": record.role_id,
        "brief_digest": record.brief_digest,
        "usage_evidence_digest": record.usage_evidence_digest,
        "state": record.state,
    });
    Ok(sha256_digest(
        &serde_json_canonicalizer::to_vec(&payload)
            .map_err(|error| AdapterError::Internal(error.to_string()))?,
    ))
}

fn publish_authorization(
    target: &PinnedTarget,
    path: &Path,
    expected: &FileSnapshot,
    record: &DispatchAuthorizationRecord,
) -> Result<bool, AdapterError> {
    let bytes = serde_json_canonicalizer::to_vec(record)
        .map_err(|error| AdapterError::Internal(error.to_string()))?;
    target.publish_runtime(path, expected, &bytes)
}

#[allow(clippy::too_many_lines)]
fn resume(arguments: &ResumeArguments) -> Result<ActionResult, AdapterError> {
    let capability_bytes =
        read_fresh_capability_file(&arguments.capabilities, MAX_EXPLICIT_FILE_BYTES)?;
    let capability = CapabilityResolution::parse_json(&capability_bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    let target = PinnedTarget::open(&arguments.target)?;
    let plan_path = run_path(&arguments.run_id, "PLAN.md")?;
    let status_path = run_path(&arguments.run_id, "STATUS.md")?;
    let plan_bytes = target.read_required(&plan_path, MAX_EXPLICIT_FILE_BYTES)?;
    let status_bytes = target.read_required(&status_path, MAX_EXPLICIT_FILE_BYTES)?;
    let plan = RunPlan::parse_markdown(&plan_bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    let status = RunStatusDocument::parse_markdown(&status_bytes)
        .map_err(|error| AdapterError::Verification(error.to_string()))?;
    status.validate_checkpoint().map_err(core_verification)?;
    if status.status().run_id != arguments.run_id {
        return Err(AdapterError::Verification(
            "STATUS.md run_id does not match requested run".to_owned(),
        ));
    }
    if as_set(plan.criteria()) != as_set(&status.status().required_criteria) {
        return Err(AdapterError::Verification(
            "PLAN.md criteria differ from STATUS.md".to_owned(),
        ));
    }
    enforce_owner_continuity(&status, &capability)?;
    let roles = load_active_roles(&target, &status.status().active_roles, &arguments.run_id)?;
    let verified_evidence = verify_checkpoint_evidence(&target, status.status())?;

    let support = status
        .owner_binding()
        .map_err(core_verification)?
        .subagent_support;
    let dispatchable = matches!(
        status.status().state,
        RunState::Executing | RunState::Verifying
    );
    if dispatchable
        && matches!(
            support,
            SupportLevel::Unsupported | SupportLevel::Unverified
        )
    {
        return Err(AdapterError::Unsupported(format!(
            "pinned subagent support is {support:?}; no dispatch brief was prepared"
        )));
    }
    let mut usage_failure = None;
    let mut usage_evidence = None;
    let mut runtime_changed_paths = Vec::new();
    let (briefs, usage_guard) = if !dispatchable {
        (
            Vec::new(),
            json!({
                "dispatch_intent": arguments.dispatch_intent.as_str(),
                "enforced": false,
                "outcome": "not_applicable",
                "evidence_digest": null,
                "window": null,
            }),
        )
    } else if arguments.dispatch_intent == DispatchIntent::Manual {
        (
            prepare_role_dispatch_briefs(&plan, &status, &roles, &capability)?,
            json!({
                "dispatch_intent": "manual",
                "enforced": false,
                "outcome": "not_requested",
                "evidence_digest": null,
                "window": null,
            }),
        )
    } else {
        let account_digest = arguments.account_digest.as_deref().ok_or_else(|| {
            AdapterError::Internal("automatic account digest was not parsed".to_owned())
        })?;
        let role_id = arguments
            .role_id
            .as_deref()
            .ok_or_else(|| AdapterError::Internal("automatic role was not parsed".to_owned()))?;
        let configured_threshold = installed_usage_threshold(&target)?;
        if arguments
            .threshold
            .is_some_and(|requested| requested != configured_threshold)
        {
            return Err(AdapterError::Input(format!(
                "--threshold must equal installed usage_stop_remaining_percent ({configured_threshold})"
            )));
        }
        let selected_role = roles
            .iter()
            .find(|loaded| loaded.role_id == role_id)
            .ok_or_else(|| {
                AdapterError::Input(format!(
                    "automatic role is not active for this run: {role_id}"
                ))
            })?;
        let brief =
            prepare_dispatch_brief(&plan, &status, &selected_role.document, Some(&capability))
                .map_err(map_dispatch_error)?;
        let (authorization_id, brief_digest, authorization_path) =
            authorization_binding(&arguments.run_id, status.status().revision, role_id, &brief)?;
        let authorization_snapshot = existing_authorization(
            &target,
            &authorization_path,
            &authorization_id,
            &arguments.run_id,
            status.status().revision,
            role_id,
            &brief_digest,
        )?;
        if authorization_snapshot.is_none() {
            usage_failure = Some((
                "hive.usage-unknown",
                "automatic dispatch authorization was already issued for this exact brief"
                    .to_owned(),
            ));
            (
                Vec::new(),
                json!({
                    "dispatch_intent": "automatic",
                    "enforced": false,
                    "outcome": "already_issued",
                    "evidence_digest": null,
                    "window": null,
                    "configured_threshold_percent": configured_threshold,
                    "history": "not_sampled",
                    "authorization_id": authorization_id,
                    "role_id": role_id,
                }),
            )
        } else {
            let Some(authorization_snapshot) = authorization_snapshot else {
                unreachable!("existing authorization is handled by the replay branch")
            };
            let (history_path, history_snapshot, previous_snapshots, history_state) =
                read_usage_history(&target, account_digest)?;
            match qualify_and_dispatch_with_runner(
                &SystemCommandRunner,
                account_digest,
                configured_threshold,
                &previous_snapshots,
                SystemTime::now(),
                current_usage_unix_seconds,
                || Ok::<DispatchBrief, AdapterError>(brief),
            ) {
                Ok(authorized) => {
                    let observation = authorized.observation;
                    let evidence = observation.evidence;
                    let brief = authorized.value?;
                    if publish_usage_history(
                        &target,
                        &history_path,
                        &history_snapshot,
                        &observation.snapshots,
                    )? {
                        runtime_changed_paths.push(portable_relative_path(&history_path));
                    }
                    let mut authorization_record = DispatchAuthorizationRecord {
                        schema_version: 1,
                        authorization_id: authorization_id.clone(),
                        run_id: arguments.run_id.clone(),
                        status_revision: status.status().revision,
                        role_id: role_id.to_owned(),
                        brief_digest,
                        usage_evidence_digest: evidence.digest.clone(),
                        state: "issued".to_owned(),
                        record_digest: String::new(),
                    };
                    authorization_record.record_digest =
                        authorization_record_digest(&authorization_record)?;
                    if publish_authorization(
                        &target,
                        &authorization_path,
                        &authorization_snapshot,
                        &authorization_record,
                    )? {
                        runtime_changed_paths.push(portable_relative_path(&authorization_path));
                    }
                    let briefs = vec![brief];
                    let data = json!({
                        "dispatch_intent": "automatic",
                        "enforced": true,
                        "outcome": "authorized",
                        "evidence_digest": evidence.digest,
                        "window": evidence.window,
                        "configured_threshold_percent": configured_threshold,
                        "history": history_state,
                        "authorization_id": authorization_id,
                        "role_id": role_id,
                    });
                    usage_evidence = Some(evidence);
                    (briefs, data)
                }
                Err(error) => {
                    if let Some(observation) = automatic_observation(&error) {
                        if publish_usage_history(
                            &target,
                            &history_path,
                            &history_snapshot,
                            &observation.snapshots,
                        )? {
                            runtime_changed_paths.push(portable_relative_path(&history_path));
                        }
                    }
                    let failure = resume_usage_failure(error);
                    let data = json!({
                        "dispatch_intent": "automatic",
                        "enforced": false,
                        "outcome": failure.outcome,
                        "evidence_digest": failure.evidence.as_ref().map(|item| item.digest.as_str()),
                        "window": failure.evidence.as_ref().map(|item| item.window),
                        "configured_threshold_percent": configured_threshold,
                        "history": history_state,
                        "authorization_id": null,
                        "role_id": role_id,
                    });
                    usage_evidence = failure.evidence;
                    usage_failure = Some((failure.code, failure.message));
                    (Vec::new(), data)
                }
            }
        }
    };
    let recovery_roles = roles
        .iter()
        .map(LoadedRole::recovery_data)
        .collect::<Result<Vec<_>, _>>()?;
    let evidence_data = verified_evidence
        .iter()
        .map(|item| {
            json!({
                "locator": item.locator,
                "digest": item.digest,
            })
        })
        .collect::<Vec<_>>();
    let plan_markdown = String::from_utf8(plan_bytes.clone())
        .map_err(|_| AdapterError::Verification("PLAN.md must be UTF-8".to_owned()))?;
    let status_body = String::from_utf8(status.body().to_vec())
        .map_err(|_| AdapterError::Verification("STATUS.md body must be UTF-8".to_owned()))?;
    let data = json!({
        "run_id": arguments.run_id,
        "state": status.status().state,
        "next_action": status.status().next_action,
        "plan_markdown": plan_markdown,
        "plan_digest": sha256_digest(&plan_bytes),
        "status": status.status(),
        "status_body": status_body,
        "status_digest": sha256_digest(&status_bytes),
        "roles": recovery_roles,
        "evidence": evidence_data,
        "dispatch_briefs": briefs,
        "recovery_only": !dispatchable || usage_failure.is_some(),
        "usage_guard": usage_guard,
        "spawned": false
    });
    let state_blocked = matches!(
        status.status().state,
        RunState::Blocked | RunState::UsageLimited
    );
    let blocked = state_blocked || usage_failure.is_some();
    let (code, message) = if let Some((code, message)) = usage_failure.as_ref() {
        (*code, message.clone())
    } else if state_blocked {
        (
            "hive.run-resume-blocked",
            "run is blocked; recovery data was loaded without dispatch".to_owned(),
        )
    } else if dispatchable {
        (
            "hive.run-resume-prepared",
            if arguments.dispatch_intent == DispatchIntent::Automatic {
                "usage-guard-authorized dispatch data prepared without spawning".to_owned()
            } else {
                "manual unenforced dispatch data prepared without spawning".to_owned()
            },
        )
    } else {
        (
            "hive.run-recovery-loaded",
            "durable recovery data loaded without a hidden transition".to_owned(),
        )
    };
    let mut result_evidence = vec![
        Evidence {
            kind: "file",
            locator: arguments.capabilities.display().to_string(),
            digest: sha256_digest(&capability_bytes),
        },
        Evidence {
            kind: "file",
            locator: portable_relative_path(&plan_path),
            digest: sha256_digest(&plan_bytes),
        },
        Evidence {
            kind: "file",
            locator: portable_relative_path(&status_path),
            digest: sha256_digest(&status_bytes),
        },
    ];
    if let Some(evidence) = usage_evidence {
        result_evidence.push(Evidence {
            kind: "report",
            locator: format!("usage-snapshots:normalized:{}", evidence.window),
            digest: evidence.digest,
        });
    }
    Ok(ActionResult {
        schema_version: 1,
        action: "ResumeWork",
        status: if blocked { "blocked" } else { "success" },
        exit_code: if blocked { 3 } else { 0 },
        code,
        message,
        changed_paths: runtime_changed_paths,
        evidence: result_evidence,
        next_action: status.status().next_action.clone(),
        data: Some(data),
    })
}

fn prepare_role_dispatch_briefs(
    plan: &RunPlan,
    status: &RunStatusDocument,
    roles: &[LoadedRole],
    capability: &CapabilityResolution,
) -> Result<Vec<DispatchBrief>, AdapterError> {
    roles
        .iter()
        .map(|loaded| {
            prepare_dispatch_brief(plan, status, &loaded.document, Some(capability))
                .map_err(map_dispatch_error)
        })
        .collect()
}

struct ResumeUsageFailure {
    code: &'static str,
    message: String,
    outcome: &'static str,
    evidence: Option<UsageGuardEvidence>,
}

fn automatic_observation(error: &AutomaticDispatchError) -> Option<&UsageObservation> {
    match error {
        AutomaticDispatchError::Blocked(observation)
        | AutomaticDispatchError::Permit(_, observation) => Some(observation),
        AutomaticDispatchError::Sensor(_)
        | AutomaticDispatchError::InvalidPolicy
        | AutomaticDispatchError::Unknown(_) => None,
    }
}

fn resume_usage_failure(error: AutomaticDispatchError) -> ResumeUsageFailure {
    match error {
        AutomaticDispatchError::Blocked(observation) => ResumeUsageFailure {
            code: "hive.usage-limited",
            message: "subscription usage is at or below the automatic dispatch threshold"
                .to_owned(),
            outcome: "limited",
            evidence: Some(observation.evidence),
        },
        AutomaticDispatchError::Sensor(error) => ResumeUsageFailure {
            code: "hive.usage-unknown",
            message: error.to_string(),
            outcome: "unknown",
            evidence: None,
        },
        AutomaticDispatchError::InvalidPolicy => ResumeUsageFailure {
            code: "hive.usage-unknown",
            message: "automatic dispatch usage policy is invalid".to_owned(),
            outcome: "unknown",
            evidence: None,
        },
        AutomaticDispatchError::Unknown(observation) => ResumeUsageFailure {
            code: "hive.usage-unknown",
            message: "subscription usage could not authorize automatic dispatch".to_owned(),
            outcome: "unknown",
            evidence: Some(observation.evidence),
        },
        AutomaticDispatchError::Permit(error, observation) => ResumeUsageFailure {
            code: "hive.usage-unknown",
            message: format!("automatic dispatch usage permit was rejected: {error:?}"),
            outcome: "unknown",
            evidence: Some(observation.evidence),
        },
    }
}

fn current_usage_unix_seconds() -> Result<i64, SensorError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .ok_or(SensorError::Malformed)
}

fn enforce_owner_continuity(
    status: &RunStatusDocument,
    capability: &CapabilityResolution,
) -> Result<(), AdapterError> {
    let binding = status.owner_binding().map_err(core_verification)?;
    match verify_owner_continuity(&binding, Some(capability)) {
        OwnerContinuity::Matched => Ok(()),
        OwnerContinuity::Blocked(reason) => Err(AdapterError::OwnerBlocked(format!(
            "run owner continuity blocked: {reason:?}"
        ))),
        OwnerContinuity::Unsupported(reason) => Err(AdapterError::OwnerUnsupported(format!(
            "run owner continuity unsupported: {reason:?}"
        ))),
    }
}

fn map_dispatch_error(error: DispatchContractError) -> AdapterError {
    match error {
        DispatchContractError::Blocked(reason) => {
            AdapterError::OwnerBlocked(format!("dispatch blocked: {reason:?}"))
        }
        DispatchContractError::Unsupported(reason) => {
            AdapterError::OwnerUnsupported(format!("dispatch unsupported: {reason:?}"))
        }
        other => AdapterError::Verification(other.to_string()),
    }
}

fn core_verification(error: impl std::fmt::Display) -> AdapterError {
    AdapterError::Verification(error.to_string())
}

pub(crate) fn run_path(run_id: &str, file_name: &str) -> Result<PathBuf, AdapterError> {
    let relative = Path::new(".hive").join("runs").join(run_id).join(file_name);
    validate_project_relative(&relative)
        .map_err(|error| AdapterError::Input(format!("unsafe run id: {error}")))?;
    Ok(relative)
}

pub(crate) fn role_path(role_id: &str) -> Result<PathBuf, AdapterError> {
    let relative = Path::new(".hive")
        .join("team")
        .join("roles")
        .join(format!("{role_id}.md"));
    validate_project_relative(&relative)
        .map_err(|error| AdapterError::Input(format!("unsafe role id: {error}")))?;
    Ok(relative)
}

pub(crate) fn portable_relative_path(path: &Path) -> String {
    path.iter()
        .map(OsStr::to_string_lossy)
        .collect::<Vec<_>>()
        .join("/")
}

struct LoadedRole {
    role_id: String,
    relative: PathBuf,
    bytes: Vec<u8>,
    document: RoleDocument,
    handoff_relative: PathBuf,
    handoff_bytes: Vec<u8>,
}

impl LoadedRole {
    fn result_evidence(&self) -> Vec<Evidence> {
        vec![
            Evidence {
                kind: "file",
                locator: portable_relative_path(&self.relative),
                digest: sha256_digest(&self.bytes),
            },
            Evidence {
                kind: "file",
                locator: portable_relative_path(&self.handoff_relative),
                digest: sha256_digest(&self.handoff_bytes),
            },
        ]
    }

    fn recovery_data(&self) -> Result<Value, AdapterError> {
        let body = String::from_utf8(self.document.body().to_vec())
            .map_err(|_| AdapterError::Verification("role body must be UTF-8".to_owned()))?;
        let handoff = crate::role::handoff_entry(
            &self.handoff_bytes,
            &self.role_id,
            self.document
                .profile()
                .current_assignment
                .as_deref()
                .ok_or_else(|| {
                    AdapterError::Verification("active role assignment is missing".to_owned())
                })?,
        )?;
        Ok(json!({
            "profile": self.document.profile(),
            "body": body,
            "body_digest": sha256_digest(self.document.body()),
            "handoff": handoff,
            "handoff_digest": sha256_digest(&self.handoff_bytes)
        }))
    }
}

fn load_active_roles(
    target: &PinnedTarget,
    role_ids: &[String],
    run_id: &str,
) -> Result<Vec<LoadedRole>, AdapterError> {
    let expected_handoff = run_path(run_id, "HANDOFF.md")?;
    let expected_handoff_text = portable_relative_path(&expected_handoff);
    let mut loaded = Vec::with_capacity(role_ids.len());
    for role_id in role_ids {
        let relative = role_path(role_id)?;
        let bytes = target.read_required(&relative, MAX_EXPLICIT_FILE_BYTES)?;
        let document = RoleDocument::parse(&bytes, role_id).map_err(core_verification)?;
        document.validate_runtime().map_err(core_verification)?;
        if document.profile().current_assignment.as_deref() != Some(run_id) {
            return Err(AdapterError::Conflict(format!(
                "active role {role_id} is not assigned to run {run_id}"
            )));
        }
        if document.profile().handoff_path.as_deref() != Some(expected_handoff_text.as_str()) {
            return Err(AdapterError::Conflict(format!(
                "active role {role_id} does not reference the exact run handoff"
            )));
        }
        let handoff_bytes = target.read_required(&expected_handoff, MAX_EXPLICIT_FILE_BYTES)?;
        crate::role::validate_handoff_document(&handoff_bytes, role_id, run_id)?;
        loaded.push(LoadedRole {
            role_id: role_id.clone(),
            relative,
            bytes,
            document,
            handoff_relative: expected_handoff.clone(),
            handoff_bytes,
        });
    }
    Ok(loaded)
}

fn verify_checkpoint_evidence(
    target: &PinnedTarget,
    status: &RunStatus,
) -> Result<Vec<Evidence>, AdapterError> {
    let mut locators = BTreeSet::new();
    locators.extend(status.latest_evidence.iter().cloned());
    for values in status.criterion_evidence.values() {
        locators.extend(values.iter().cloned());
    }
    let mut total = 0_usize;
    let mut verified = Vec::with_capacity(locators.len());
    for locator in locators {
        let (path, expected_digest) = locator.split_once('#').ok_or_else(|| {
            AdapterError::Verification(format!("evidence locator lacks digest: {locator}"))
        })?;
        let relative = Path::new(path);
        let bytes = target.read_required(relative, MAX_EVIDENCE_FILE_BYTES)?;
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| AdapterError::Verification("evidence byte count overflow".to_owned()))?;
        if total > MAX_TOTAL_EVIDENCE_BYTES {
            return Err(AdapterError::Verification(
                "verified evidence exceeds the bounded aggregate".to_owned(),
            ));
        }
        let actual = sha256_digest(&bytes);
        if actual != expected_digest {
            return Err(AdapterError::Verification(format!(
                "evidence digest mismatch at {path}"
            )));
        }
        verified.push(Evidence {
            kind: "file",
            locator,
            digest: actual,
        });
    }
    Ok(verified)
}

fn as_set(values: &[String]) -> BTreeSet<String> {
    values.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::{
        checkpoint, parse_resume_arguments, publish_parent_file, publish_parent_file_with_hook,
        publish_parent_file_with_hooks, read_explicit_file_with_metadata_and_hooks, resume,
        run_run, CheckpointArguments, DispatchIntent, FileSnapshot, ResumeArguments,
    };
    #[cfg(unix)]
    use super::{read_explicit_file, PinnedTarget};
    use cap_std::ambient_authority;
    use cap_std::fs::Dir;
    use hive_core::sha256_digest;
    use serde_json::json;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    const CAPABILITY: &[u8] =
        include_bytes!("../../../tests/fixtures/phase1/capabilities-codex-omx.json");
    const ABSENT_CAPABILITY: &[u8] =
        include_bytes!("../../../tests/fixtures/phase1/capabilities-absent.json");

    fn canonical(path: &Path) -> PathBuf {
        path.canonicalize().expect("canonical path")
    }

    fn role_bytes(role_id: &str) -> Vec<u8> {
        format!(
            "---\n{{\"allowed_capabilities\":[\"filesystem-read\"],\"context_paths\":[\"docs/\"],\"current_assignment\":\"run-1\",\"display_name\":\"Role\",\"handoff_path\":\".hive/runs/run-1/HANDOFF.md\",\"non_responsibilities\":[\"other\"],\"responsibilities\":[\"work\"],\"role_id\":\"{role_id}\",\"schema_version\":1,\"verification_duties\":[\"verify\"],\"write_scope\":[\".hive/runs/\"]}}\n---\n# {role_id}\n"
        )
        .into_bytes()
    }

    fn shared_handoff(role_id: &str) -> Vec<u8> {
        let mut handoffs = serde_json::Map::new();
        handoffs.insert(
            role_id.to_owned(),
            json!({
                "markdown": "continue",
                "updated_at": "2026-07-24T00:00:00Z"
            }),
        );
        let value = json!({
            "schema_version": 1,
            "run_id": "run-1",
            "updated_at": "2026-07-24T00:00:00Z",
            "handoffs": handoffs
        });
        let frontmatter =
            serde_json_canonicalizer::to_string(&value).expect("canonical handoff JSON");
        format!("---\n{frontmatter}\n---\n# Role handoffs\n").into_bytes()
    }

    fn setup_run() -> (TempDir, PathBuf, PathBuf) {
        let temp = TempDir::new().expect("temporary consumer");
        let target = canonical(temp.path());
        fs::create_dir_all(target.join(".hive/team/roles")).expect("role directory");
        fs::create_dir_all(target.join(".hive/runs/run-1/evidence")).expect("run directory");
        fs::write(
            target.join(".hive/runs/run-1/PLAN.md"),
            "# Plan\n\n- [ ] [build] build succeeds\n- [ ] [tests] tests pass\n",
        )
        .expect("plan");
        fs::write(
            target.join(".hive/team/roles/reviewer.md"),
            role_bytes("reviewer"),
        )
        .expect("role");
        fs::write(
            target.join(".hive/runs/run-1/HANDOFF.md"),
            shared_handoff("reviewer"),
        )
        .expect("handoff");
        let capability = target.join("capability.json");
        fs::write(&capability, CAPABILITY).expect("capability");
        (temp, target, capability)
    }

    fn write_checkpoint_request(
        target: &Path,
        expected_revision: u64,
        updated_at: &str,
        passed: &[&str],
        evidence: Option<&str>,
    ) -> PathBuf {
        let passed = passed.to_vec();
        let criterion_evidence =
            evidence.map_or_else(|| json!({}), |locator| json!({"build": [locator]}));
        let latest_evidence = evidence.map_or_else(Vec::new, |locator| vec![locator]);
        let request = json!({
            "schema_version": 1,
            "run_id": "run-1",
            "expected_revision": expected_revision,
            "state": "executing",
            "passed_criteria": passed,
            "failed_criteria": [],
            "active_roles": ["reviewer"],
            "next_action": "continue",
            "latest_evidence": latest_evidence,
            "blocker": null,
            "resume_note": null,
            "criterion_evidence": criterion_evidence,
            "updated_at": updated_at
        });
        let timestamp_digest = sha256_digest(updated_at.as_bytes());
        let path = target.join(format!(
            "checkpoint-{expected_revision}-{}.json",
            &timestamp_digest[7..15]
        ));
        fs::write(
            &path,
            serde_json::to_vec(&request).expect("checkpoint JSON"),
        )
        .expect("checkpoint request");
        path
    }

    fn write_custom_checkpoint_request(
        target: &Path,
        state: &str,
        blocker: Option<&str>,
    ) -> PathBuf {
        let request = json!({
            "schema_version": 1,
            "run_id": "run-1",
            "expected_revision": 0,
            "state": state,
            "passed_criteria": [],
            "failed_criteria": [],
            "active_roles": ["reviewer"],
            "next_action": "continue",
            "latest_evidence": [],
            "blocker": blocker,
            "resume_note": null,
            "criterion_evidence": {},
            "updated_at": "2026-07-24T00:00:00Z"
        });
        let path = target.join(format!("checkpoint-{state}.json"));
        fs::write(
            &path,
            serde_json::to_vec(&request).expect("checkpoint JSON"),
        )
        .expect("checkpoint request");
        path
    }

    fn write_unsupported_capability(target: &Path) -> PathBuf {
        let mut value: serde_json::Value =
            serde_json::from_slice(ABSENT_CAPABILITY).expect("absent capability");
        value["capabilities"]["subagents"] = json!("unsupported");
        value
            .as_object_mut()
            .expect("capability object")
            .remove("evidence_digest");
        let canonical =
            serde_json_canonicalizer::to_vec(&value).expect("canonical capability body");
        value["evidence_digest"] = json!(sha256_digest(&canonical));
        let path = target.join("unsupported-capability.json");
        fs::write(
            &path,
            serde_json::to_vec(&value).expect("unsupported capability"),
        )
        .expect("capability");
        path
    }

    #[test]
    fn atomic_publication_creates_updates_and_rejects_stale_bytes() {
        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        let directory =
            Dir::open_ambient_dir(&root, ambient_authority()).expect("capability directory");
        publish_parent_file(
            &directory,
            OsStr::new("status"),
            &FileSnapshot::Missing,
            b"one",
        )
        .expect("create");
        assert_eq!(fs::read(root.join("status")).expect("created"), b"one");
        publish_parent_file(
            &directory,
            OsStr::new("status"),
            &FileSnapshot::File(b"one".to_vec()),
            b"two",
        )
        .expect("update");
        assert_eq!(fs::read(root.join("status")).expect("updated"), b"two");
        let error = publish_parent_file(
            &directory,
            OsStr::new("status"),
            &FileSnapshot::File(b"one".to_vec()),
            b"three",
        )
        .expect_err("stale bytes");
        assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
        assert_eq!(fs::read(root.join("status")).expect("preserved"), b"two");
    }

    #[test]
    fn atomic_publication_preserves_racing_destination_and_recovery_copy() {
        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        fs::write(root.join("status"), b"prior").expect("prior");
        let directory =
            Dir::open_ambient_dir(&root, ambient_authority()).expect("capability directory");
        let error = publish_parent_file_with_hook(
            &directory,
            OsStr::new("status"),
            &FileSnapshot::File(b"prior".to_vec()),
            b"desired",
            |parent, destination| parent.write(destination, b"racer"),
        )
        .expect_err("racer blocks exclusive publication");
        assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
        assert_eq!(fs::read(root.join("status")).expect("racer"), b"racer");
        let recovered = fs::read_dir(&root)
            .expect("root entries")
            .filter_map(Result::ok)
            .find_map(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".aigent-hive-run-claim-")
                    .then(|| fs::read(entry.path().join("claimed")).ok())
                    .flatten()
            });
        assert_eq!(recovered.as_deref(), Some(b"prior".as_slice()));
    }

    #[test]
    fn failed_new_file_cleanup_rolls_back_exact_publish_and_retains_recovery_link() {
        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        let directory =
            Dir::open_ambient_dir(&root, ambient_authority()).expect("capability directory");
        let error = publish_parent_file_with_hooks(
            &directory,
            OsStr::new("status"),
            &FileSnapshot::Missing,
            b"desired",
            |_, _| Ok(()),
            |_, _| Err(std::io::Error::other("injected unlink failure")),
        )
        .expect_err("cleanup failure reported");
        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        assert!(
            !root.join("status").exists(),
            "failed first publication must leave canonical write-zero"
        );
        let recovery = fs::read_dir(&root)
            .expect("root entries")
            .filter_map(Result::ok)
            .find_map(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".aigent-hive-run-")
                    .then(|| fs::read(entry.path()).ok())
                    .flatten()
            });
        assert_eq!(recovery.as_deref(), Some(b"desired".as_slice()));
    }

    #[test]
    fn failed_new_file_cleanup_preserves_racing_destination() {
        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        let directory =
            Dir::open_ambient_dir(&root, ambient_authority()).expect("capability directory");
        publish_parent_file_with_hooks(
            &directory,
            OsStr::new("status"),
            &FileSnapshot::Missing,
            b"desired",
            |_, _| Ok(()),
            |parent, _| {
                parent.remove_file("status")?;
                parent.write("status", b"racer")?;
                Err(std::io::Error::other("injected cleanup race"))
            },
        )
        .expect_err("cleanup race reported");
        assert_eq!(fs::read(root.join("status")).expect("racer"), b"racer");
        let desired_recovery = fs::read_dir(&root)
            .expect("root entries")
            .filter_map(Result::ok)
            .find_map(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".aigent-hive-run-")
                    .then(|| fs::read(entry.path()).ok())
                    .flatten()
                    .filter(|bytes| bytes == b"desired")
            });
        assert_eq!(desired_recovery.as_deref(), Some(b"desired".as_slice()));
    }

    #[test]
    fn failed_update_cleanup_rolls_back_prior_bytes() {
        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        fs::write(root.join("status"), b"prior").expect("prior");
        let directory =
            Dir::open_ambient_dir(&root, ambient_authority()).expect("capability directory");
        publish_parent_file_with_hooks(
            &directory,
            OsStr::new("status"),
            &FileSnapshot::File(b"prior".to_vec()),
            b"desired",
            |_, _| Ok(()),
            |_, _| Err(std::io::Error::other("injected unlink failure")),
        )
        .expect_err("cleanup failure reported");
        assert_eq!(fs::read(root.join("status")).expect("restored"), b"prior");
    }

    #[test]
    fn resume_dispatch_intent_options_are_additive_and_automatic_only() {
        let base = [
            "--target",
            "/consumer",
            "--run",
            "run-1",
            "--capabilities",
            "/capabilities.json",
            "--output",
            "json",
        ]
        .map(str::to_owned);
        let manual = parse_resume_arguments(&base).expect("default manual intent");
        assert_eq!(manual.dispatch_intent, DispatchIntent::Manual);
        assert_eq!(manual.account_digest, None);
        assert_eq!(manual.role_id, None);
        assert_eq!(manual.threshold, None);

        let mut automatic = base.to_vec();
        automatic.extend(
            [
                "--dispatch-intent",
                "automatic",
                "--account-digest",
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--role",
                "reviewer",
                "--threshold",
                "17",
            ]
            .map(str::to_owned),
        );
        let automatic = parse_resume_arguments(&automatic).expect("automatic intent");
        assert_eq!(automatic.dispatch_intent, DispatchIntent::Automatic);
        assert_eq!(automatic.role_id.as_deref(), Some("reviewer"));
        assert_eq!(automatic.threshold, Some(17));

        let mut missing_account = base.to_vec();
        missing_account.extend(["--dispatch-intent", "automatic"].map(str::to_owned));
        assert!(parse_resume_arguments(&missing_account).is_err());

        let mut manual_threshold = base.to_vec();
        manual_threshold.extend(["--threshold", "17"].map(str::to_owned));
        assert!(parse_resume_arguments(&manual_threshold).is_err());
    }

    #[test]
    fn checkpoint_is_idempotent_and_lost_revision_is_rejected() {
        let (_temp, target, capability) = setup_run();
        let request = write_checkpoint_request(&target, 0, "2026-07-24T00:00:00Z", &[], None);
        let arguments = || CheckpointArguments {
            target: target.clone(),
            request: request.clone(),
            capabilities: capability.clone(),
        };
        let first = checkpoint(&arguments()).expect("checkpoint");
        assert_eq!(first.status, "success");
        let status_path = target.join(".hive/runs/run-1/STATUS.md");
        let first_bytes = fs::read(&status_path).expect("status");
        let retry = checkpoint(&arguments()).expect("exact retry");
        assert_eq!(retry.code, "hive.run-checkpoint-idempotent");
        assert_eq!(fs::read(&status_path).expect("status"), first_bytes);

        let lost = write_checkpoint_request(&target, 0, "2026-07-24T00:01:00Z", &[], None);
        let error = checkpoint(&CheckpointArguments {
            target,
            request: lost,
            capabilities: capability,
        })
        .err()
        .expect("lost revision rejected");
        assert_eq!(error.status(), "conflict");
    }

    #[test]
    fn checkpoint_verifies_exact_evidence_bytes_and_resume_is_read_only() {
        let (_temp, target, capability) = setup_run();
        let evidence_path = target.join(".hive/runs/run-1/evidence/build.json");
        fs::write(&evidence_path, b"{\"ok\":true}\n").expect("evidence");
        let locator = format!(
            ".hive/runs/run-1/evidence/build.json#{}",
            sha256_digest(b"{\"ok\":true}\n")
        );
        let request = write_checkpoint_request(
            &target,
            0,
            "2026-07-24T00:00:00Z",
            &["build"],
            Some(&locator),
        );
        checkpoint(&CheckpointArguments {
            target: target.clone(),
            request,
            capabilities: capability.clone(),
        })
        .expect("checkpoint with evidence");
        let status_before =
            fs::read(target.join(".hive/runs/run-1/STATUS.md")).expect("status before");
        let resumed = resume(&ResumeArguments {
            target: target.clone(),
            run_id: "run-1".to_owned(),
            capabilities: capability.clone(),
            dispatch_intent: DispatchIntent::Manual,
            account_digest: None,
            role_id: None,
            threshold: None,
        })
        .expect("resume prepares briefs");
        assert_eq!(resumed.code, "hive.run-resume-prepared");
        assert_eq!(
            resumed.data.as_ref().and_then(|data| data.get("spawned")),
            Some(&json!(false))
        );
        assert_eq!(
            fs::read(target.join(".hive/runs/run-1/STATUS.md")).expect("status after"),
            status_before
        );

        fs::write(&evidence_path, b"{\"ok\":false}\n").expect("tamper");
        let error = resume(&ResumeArguments {
            target,
            run_id: "run-1".to_owned(),
            capabilities: capability,
            dispatch_intent: DispatchIntent::Manual,
            account_digest: None,
            role_id: None,
            threshold: None,
        })
        .err()
        .expect("tamper rejected");
        assert_eq!(error.status(), "verification-failed");
    }

    #[test]
    fn resume_rejects_owner_drift_and_unsupported_dispatch_without_briefs() {
        let (_temp, target, capability) = setup_run();
        let request = write_checkpoint_request(&target, 0, "2026-07-24T00:00:00Z", &[], None);
        checkpoint(&CheckpointArguments {
            target: target.clone(),
            request,
            capabilities: capability,
        })
        .expect("OMX checkpoint");
        let absent_path = target.join("absent.json");
        fs::write(&absent_path, ABSENT_CAPABILITY).expect("absent capability");
        let drift = resume(&ResumeArguments {
            target: target.clone(),
            run_id: "run-1".to_owned(),
            capabilities: absent_path,
            dispatch_intent: DispatchIntent::Manual,
            account_digest: None,
            role_id: None,
            threshold: None,
        })
        .err()
        .expect("owner drift blocks");
        assert!(matches!(
            drift,
            super::AdapterError::OwnerBlocked(_) | super::AdapterError::OwnerUnsupported(_)
        ));

        fs::remove_file(target.join(".hive/runs/run-1/STATUS.md")).expect("reset status");
        let unsupported = write_unsupported_capability(&target);
        let request = write_checkpoint_request(&target, 0, "2026-07-24T00:02:00Z", &[], None);
        checkpoint(&CheckpointArguments {
            target: target.clone(),
            request,
            capabilities: unsupported.clone(),
        })
        .expect("unsupported support may checkpoint");
        let error = resume(&ResumeArguments {
            target,
            run_id: "run-1".to_owned(),
            capabilities: unsupported,
            dispatch_intent: DispatchIntent::Manual,
            account_digest: None,
            role_id: None,
            threshold: None,
        })
        .err()
        .expect("unsupported dispatch rejected");
        assert_eq!(error.exit_code(), 4);
    }

    #[test]
    fn blocked_resume_returns_recovery_without_dispatch() {
        let (_temp, target, capability) = setup_run();
        let request =
            write_custom_checkpoint_request(&target, "blocked", Some("manual approval required"));
        checkpoint(&CheckpointArguments {
            target: target.clone(),
            request,
            capabilities: capability.clone(),
        })
        .expect("blocked checkpoint");
        let result = resume(&ResumeArguments {
            target,
            run_id: "run-1".to_owned(),
            capabilities: capability,
            dispatch_intent: DispatchIntent::Manual,
            account_digest: None,
            role_id: None,
            threshold: None,
        })
        .expect("blocked recovery");
        assert_eq!(result.status, "blocked");
        assert_eq!(result.exit_code, 3);
        let data = result.data.expect("recovery data");
        assert_eq!(data["dispatch_briefs"], json!([]));
        assert_eq!(data["recovery_only"], json!(true));
        assert_eq!(data["spawned"], json!(false));
    }

    #[test]
    fn explicit_input_rejects_identity_swap_after_preflight() {
        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        let input = root.join("input.json");
        fs::write(&input, b"original").expect("input");
        fs::write(root.join("replacement.json"), b"replacement").expect("replacement");

        let error = read_explicit_file_with_metadata_and_hooks(
            &input,
            1024,
            |parent, name| {
                parent.remove_file(name)?;
                parent.rename("replacement.json", parent, name)
            },
            |_, _| Ok(()),
        )
        .expect_err("replacement inode must be rejected");

        assert!(
            error
                .message()
                .contains("changed between preflight and open"),
            "{}",
            error.message()
        );
        assert_eq!(
            fs::read(input).expect("replacement preserved"),
            b"replacement"
        );
    }

    #[test]
    fn explicit_input_rejects_same_handle_mutation_during_read() {
        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        let input = root.join("input.json");
        fs::write(&input, b"original").expect("input");

        let error = read_explicit_file_with_metadata_and_hooks(
            &input,
            1024,
            |_, _| Ok(()),
            |parent, name| parent.write(name, b"changed-and-longer"),
        )
        .expect_err("same inode mutation must be rejected");

        assert!(
            error.message().contains("changed while it was read"),
            "{}",
            error.message()
        );
    }

    #[cfg(unix)]
    #[test]
    fn explicit_input_fifo_swap_is_nonblocking_and_rejected() {
        use std::process::Command;
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        let input = root.join("input.json");
        fs::write(&input, b"original").expect("input");
        let worker_input = input.clone();
        let (sender, receiver) = mpsc::channel();

        let worker = thread::spawn(move || {
            let fifo_path = worker_input.clone();
            let outcome = read_explicit_file_with_metadata_and_hooks(
                &worker_input,
                1024,
                move |parent, name| {
                    parent.remove_file(name)?;
                    let status = Command::new("mkfifo").arg(&fifo_path).status()?;
                    if status.success() {
                        Ok(())
                    } else {
                        Err(std::io::Error::other(format!(
                            "mkfifo exited with {status}"
                        )))
                    }
                },
                |_, _| Ok(()),
            )
            .map(|_| ())
            .map_err(|error| error.message().to_owned());
            let _ = sender.send(outcome);
        });

        let outcome = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("nonblocking open must reject a FIFO swap within five seconds");
        worker.join().expect("reader thread");
        let error = outcome.expect_err("FIFO must be rejected");
        assert!(error.contains("regular file"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn nofollow_inputs_and_pinned_target_reject_symlink_swaps() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().expect("temp");
        let root = canonical(temp.path());
        let real = root.join("real.json");
        let alias = root.join("alias.json");
        fs::write(&real, b"{}").expect("real");
        symlink(&real, &alias).expect("input symlink");
        assert!(read_explicit_file(&alias, 1024).is_err());

        let target = root.join("consumer");
        fs::create_dir(&target).expect("target");
        let pinned = PinnedTarget::open(&target).expect("pin target");
        let moved = root.join("moved");
        fs::rename(&target, &moved).expect("move target");
        symlink(&moved, &target).expect("swap target");
        let error = pinned
            .snapshot(Path::new(".hive/runs/run-1/STATUS.md"))
            .expect("pinned handle remains readable");
        assert!(matches!(error, FileSnapshot::Missing));
        assert!(pinned
            .publish(
                Path::new(".hive/runs/run-1/STATUS.md"),
                &FileSnapshot::Missing,
                b"new"
            )
            .is_err());
    }

    #[test]
    fn run_action_help_is_read_only_and_successful() {
        for action in ["checkpoint", "resume"] {
            assert_eq!(
                run_run(&[action.to_owned(), "--help".to_owned()]),
                std::process::ExitCode::SUCCESS
            );
        }
    }
}
