//! No-follow, bounded filesystem I/O for complete `.hivekb` archives.

use crate::portable::{
    decode_bundle, encode_bundle, BundleLimits, BundleRequest, PortableError, ValidatedBundlePlan,
};
use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TRANSACTION_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const TRANSACTION_ATTEMPTS: usize = 128;

/// Existing-destination policy for bundle publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BundlePublishMode {
    /// Create a new bundle, or return an exact-byte no-op for the same bundle.
    CreateOnly,
    /// Replace different bytes while retaining the prior file under this sibling backup name.
    Replace {
        /// One normal filename in the destination directory. It must not already exist.
        backup_file_name: OsString,
    },
}

/// Durable publication result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundlePublishOutcome {
    /// A previously missing destination was created.
    Created,
    /// The destination already contained the exact deterministic archive bytes.
    Unchanged,
    /// Different prior bytes were replaced and retained as an explicit sibling backup.
    Replaced,
}

/// Receipt for one completed bundle publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundlePublishReceipt {
    outcome: BundlePublishOutcome,
    archive_sha256: String,
    byte_length: u64,
    backup_path: Option<PathBuf>,
}

impl BundlePublishReceipt {
    /// Publication disposition.
    #[must_use]
    pub const fn outcome(&self) -> BundlePublishOutcome {
        self.outcome
    }

    /// SHA-256 digest of the exact deterministic archive bytes.
    #[must_use]
    pub fn archive_sha256(&self) -> &str {
        &self.archive_sha256
    }

    /// Exact archive byte length.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    /// Retained prior bundle path for a successful replacement.
    #[must_use]
    pub fn backup_path(&self) -> Option<&Path> {
        self.backup_path.as_deref()
    }
}

/// Bundle filesystem boundary failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BundleIoError {
    /// Invalid destination, backup name, or filesystem limit.
    InvalidInput(String),
    /// A no-follow object or concurrent filesystem state conflicted with the operation.
    Conflict(String),
    /// A filesystem durability operation failed.
    Io(String),
    /// Portable archive construction or validation failed.
    Portable(PortableError),
}

impl BundleIoError {
    /// Stable machine-facing error family.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "hive.bundle-io-invalid-input",
            Self::Conflict(_) => "hive.bundle-io-conflict",
            Self::Io(_) => "hive.bundle-io-failed",
            Self::Portable(error) => error.code(),
        }
    }
}

impl Display for BundleIoError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) | Self::Conflict(message) | Self::Io(message) => {
                formatter.write_str(message)
            }
            Self::Portable(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for BundleIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Portable(error) => Some(error),
            Self::InvalidInput(_) | Self::Conflict(_) | Self::Io(_) => None,
        }
    }
}

impl From<PortableError> for BundleIoError {
    fn from(error: PortableError) -> Self {
        Self::Portable(error)
    }
}

/// Encode and durably publish one deterministic bundle to an explicit `.hivekb` path.
///
/// The destination directory is traversed and pinned without following symlinks. Publication uses
/// a synced sibling staging file. Different existing bytes require [`BundlePublishMode::Replace`]
/// and leave the prior bytes at the explicit backup name.
///
/// # Errors
///
/// Returns an error for invalid bundle input, unsafe paths, non-regular objects, resource limits,
/// concurrent replacement, an occupied backup, or failed durability/rollback operations.
pub fn encode_and_publish_bundle(
    request: &BundleRequest,
    limits: BundleLimits,
    destination: &Path,
    mode: &BundlePublishMode,
) -> Result<BundlePublishReceipt, BundleIoError> {
    let encoded = encode_bundle(request, limits)?;
    let archive_sha256 = encoded.plan().archive_sha256().to_owned();
    let byte_length = u64::try_from(encoded.archive().len())
        .map_err(|_| BundleIoError::InvalidInput("archive length does not fit u64".to_owned()))?;
    let pinned = PinnedDestination::open(destination)?;
    let existing = read_optional_regular(
        &pinned.parent,
        &pinned.name,
        limits.max_archive_bytes,
        "bundle destination",
    )?;
    if existing
        .as_ref()
        .is_some_and(|(_, bytes)| bytes == encoded.archive())
    {
        return Ok(BundlePublishReceipt {
            outcome: BundlePublishOutcome::Unchanged,
            archive_sha256,
            byte_length,
            backup_path: None,
        });
    }
    let expected = existing.as_ref().map(|(identity, _)| *identity);
    let backup = replacement_backup(mode, expected, &pinned)?;
    let names = unique_transaction_names(&pinned.parent, backup.as_deref())?;
    let staged = write_synced_staging(&pinned.parent, &names.staging, encoded.archive())?;
    let result = publish_staged(&pinned, &names, staged, expected, backup.as_deref());
    if result.is_err() {
        remove_if_identity(&pinned.parent, &names.staging, staged);
    }
    let outcome = result?;
    Ok(BundlePublishReceipt {
        outcome,
        archive_sha256,
        byte_length,
        backup_path: backup.map(|name| pinned.parent_path.join(name)),
    })
}

/// Load and validate an explicit bundle without resolving or extracting any payload path.
///
/// The complete file is opened no-follow, bounded by `limits.max_archive_bytes` before allocation,
/// read to its exact inspected length, and then passed to the extraction-free portable decoder.
///
/// # Errors
///
/// Returns an error for unsafe paths, missing/non-regular files, size or read races, I/O failures,
/// or any portable archive validation failure.
pub fn load_bundle(
    source: &Path,
    limits: BundleLimits,
) -> Result<ValidatedBundlePlan, BundleIoError> {
    if limits.max_archive_bytes == 0 {
        return Err(BundleIoError::InvalidInput(
            "bundle archive size limit must be positive".to_owned(),
        ));
    }
    let pinned = PinnedDestination::open(source)?;
    let (_, bytes) = read_optional_regular(
        &pinned.parent,
        &pinned.name,
        limits.max_archive_bytes,
        "bundle source",
    )?
    .ok_or_else(|| BundleIoError::InvalidInput("bundle source does not exist".to_owned()))?;
    decode_bundle(&bytes, limits).map_err(BundleIoError::from)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObjectIdentity {
    dev: u64,
    ino: u64,
}

impl ObjectIdentity {
    fn from_metadata(metadata: &cap_std::fs::Metadata) -> Self {
        Self {
            dev: CapMetadataExt::dev(metadata),
            ino: CapMetadataExt::ino(metadata),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    object: ObjectIdentity,
    length: u64,
}

impl FileIdentity {
    fn from_metadata(metadata: &cap_std::fs::Metadata) -> Self {
        Self {
            object: ObjectIdentity::from_metadata(metadata),
            length: metadata.len(),
        }
    }
}

struct PinnedDestination {
    parent: Dir,
    parent_path: PathBuf,
    parent_identity: ObjectIdentity,
    name: OsString,
}

impl PinnedDestination {
    fn open(path: &Path) -> Result<Self, BundleIoError> {
        let absolute = normalized_absolute(path)?;
        if absolute.extension() != Some(OsStr::new("hivekb")) {
            return Err(BundleIoError::InvalidInput(
                "bundle path must use the exact .hivekb extension".to_owned(),
            ));
        }
        let name = absolute
            .file_name()
            .ok_or_else(|| BundleIoError::InvalidInput("bundle path has no filename".to_owned()))?
            .to_os_string();
        let parent_path = absolute
            .parent()
            .ok_or_else(|| BundleIoError::InvalidInput("bundle path has no parent".to_owned()))?
            .to_path_buf();
        let (parent, parent_identity) = open_directory_nofollow(&parent_path)?;
        Ok(Self {
            parent,
            parent_path,
            parent_identity,
            name,
        })
    }

    fn verify_binding(&self) -> Result<(), BundleIoError> {
        let (_, current) = open_directory_nofollow(&self.parent_path)?;
        if current != self.parent_identity {
            return Err(BundleIoError::Conflict(
                "bundle destination parent changed while pinned".to_owned(),
            ));
        }
        Ok(())
    }
}

fn normalized_absolute(path: &Path) -> Result<PathBuf, BundleIoError> {
    if path.as_os_str().is_empty() {
        return Err(BundleIoError::InvalidInput(
            "bundle path cannot be empty".to_owned(),
        ));
    }
    let input = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| BundleIoError::Io(format!("cannot read current directory: {error}")))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in input.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(BundleIoError::InvalidInput(
                        "bundle path escapes its filesystem root".to_owned(),
                    ));
                }
            }
            Component::Normal(value) => normalized.push(value),
        }
    }
    if !normalized.is_absolute() {
        return Err(BundleIoError::InvalidInput(
            "bundle path could not be normalized absolutely".to_owned(),
        ));
    }
    Ok(normalized)
}

fn open_directory_nofollow(path: &Path) -> Result<(Dir, ObjectIdentity), BundleIoError> {
    let mut anchor = PathBuf::new();
    let mut descendants = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => anchor.push(prefix.as_os_str()),
            Component::RootDir => anchor.push(component.as_os_str()),
            Component::Normal(value) => descendants.push(value.to_os_string()),
            Component::CurDir | Component::ParentDir => {
                return Err(BundleIoError::InvalidInput(
                    "bundle parent is not lexically normalized".to_owned(),
                ));
            }
        }
    }
    if anchor.as_os_str().is_empty() {
        return Err(BundleIoError::InvalidInput(
            "bundle parent has no absolute filesystem anchor".to_owned(),
        ));
    }
    let mut current = Dir::open_ambient_dir(&anchor, ambient_authority())
        .map_err(|error| BundleIoError::Io(format!("cannot open filesystem root: {error}")))?;
    for component in descendants {
        let expected = current.symlink_metadata(&component).map_err(|error| {
            BundleIoError::Io(format!("cannot inspect bundle parent component: {error}"))
        })?;
        if !expected.is_dir() {
            return Err(BundleIoError::Conflict(
                "bundle parent contains a symlink or non-directory component".to_owned(),
            ));
        }
        let next = current.open_dir_nofollow(&component).map_err(|error| {
            BundleIoError::Conflict(format!(
                "cannot pin bundle parent component no-follow: {error}"
            ))
        })?;
        let actual = next.dir_metadata().map_err(|error| {
            BundleIoError::Io(format!("cannot inspect pinned bundle parent: {error}"))
        })?;
        if ObjectIdentity::from_metadata(&expected) != ObjectIdentity::from_metadata(&actual) {
            return Err(BundleIoError::Conflict(
                "bundle parent changed while its capability was pinned".to_owned(),
            ));
        }
        current = next;
    }
    let metadata = current.dir_metadata().map_err(|error| {
        BundleIoError::Io(format!("cannot inspect bundle destination parent: {error}"))
    })?;
    Ok((current, ObjectIdentity::from_metadata(&metadata)))
}

fn replacement_backup(
    mode: &BundlePublishMode,
    existing: Option<FileIdentity>,
    pinned: &PinnedDestination,
) -> Result<Option<OsString>, BundleIoError> {
    match (existing, mode) {
        (Some(_), BundlePublishMode::CreateOnly) => Err(BundleIoError::Conflict(
            "different bundle bytes already exist; explicit replace mode is required".to_owned(),
        )),
        (Some(_), BundlePublishMode::Replace { backup_file_name }) => {
            validate_backup_name(backup_file_name, &pinned.name)?;
            if entry_metadata(&pinned.parent, backup_file_name)?.is_some() {
                return Err(BundleIoError::Conflict(
                    "explicit bundle backup path is already occupied".to_owned(),
                ));
            }
            Ok(Some(backup_file_name.clone()))
        }
        (None, BundlePublishMode::CreateOnly) => Ok(None),
        (None, BundlePublishMode::Replace { backup_file_name }) => {
            validate_backup_name(backup_file_name, &pinned.name)?;
            Ok(None)
        }
    }
}

fn validate_backup_name(name: &OsStr, destination: &OsStr) -> Result<(), BundleIoError> {
    let components = Path::new(name).components().collect::<Vec<_>>();
    if components.as_slice() != [Component::Normal(name)] || name == destination {
        return Err(BundleIoError::InvalidInput(
            "bundle backup must be a distinct sibling filename".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct TransactionNames {
    staging: OsString,
    prior_claim: OsString,
    failed_claim: OsString,
}

fn unique_transaction_names(
    parent: &Dir,
    backup: Option<&OsStr>,
) -> Result<TransactionNames, BundleIoError> {
    for _ in 0..TRANSACTION_ATTEMPTS {
        let suffix = format!(
            "{}-{}",
            std::process::id(),
            TRANSACTION_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let names = TransactionNames {
            staging: OsString::from(format!(".hivekb-stage-{suffix}")),
            prior_claim: OsString::from(format!(".hivekb-prior-{suffix}")),
            failed_claim: OsString::from(format!(".hivekb-failed-{suffix}")),
        };
        if backup.is_some_and(|value| {
            value == names.staging || value == names.prior_claim || value == names.failed_claim
        }) {
            continue;
        }
        if entry_metadata(parent, &names.staging)?.is_none()
            && entry_metadata(parent, &names.prior_claim)?.is_none()
            && entry_metadata(parent, &names.failed_claim)?.is_none()
        {
            return Ok(names);
        }
    }
    Err(BundleIoError::Conflict(
        "cannot reserve bounded sibling bundle transaction names".to_owned(),
    ))
}

fn entry_metadata(
    parent: &Dir,
    name: &OsStr,
) -> Result<Option<cap_std::fs::Metadata>, BundleIoError> {
    match parent.symlink_metadata(name) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(BundleIoError::Io(format!(
            "cannot inspect sibling bundle path: {error}"
        ))),
    }
}

fn regular_identity(
    parent: &Dir,
    name: &OsStr,
    context: &str,
) -> Result<Option<FileIdentity>, BundleIoError> {
    match entry_metadata(parent, name)? {
        None => Ok(None),
        Some(metadata) if metadata.is_file() => Ok(Some(FileIdentity::from_metadata(&metadata))),
        Some(_) => Err(BundleIoError::Conflict(format!(
            "{context} must be a regular no-follow file"
        ))),
    }
}

fn read_optional_regular(
    parent: &Dir,
    name: &OsStr,
    max_bytes: u64,
    context: &str,
) -> Result<Option<(FileIdentity, Vec<u8>)>, BundleIoError> {
    let Some(expected) = regular_identity(parent, name, context)? else {
        return Ok(None);
    };
    if expected.length > max_bytes {
        return Err(BundleIoError::Portable(PortableError::BudgetExceeded(
            format!("{context} exceeds the archive-size limit before read"),
        )));
    }
    let capacity = usize::try_from(expected.length).map_err(|_| {
        BundleIoError::Portable(PortableError::BudgetExceeded(format!(
            "{context} length does not fit memory"
        )))
    })?;
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = parent.open_with(name, &options).map_err(|error| {
        BundleIoError::Conflict(format!("cannot open {context} no-follow: {error}"))
    })?;
    let opened = file
        .metadata()
        .map_err(|error| BundleIoError::Io(format!("cannot inspect opened {context}: {error}")))?;
    if FileIdentity::from_metadata(&opened) != expected {
        return Err(BundleIoError::Conflict(format!(
            "{context} changed while opened no-follow"
        )));
    }
    let mut bytes = vec![0_u8; capacity];
    file.read_exact(&mut bytes).map_err(|error| {
        BundleIoError::Conflict(format!("cannot read exact {context}: {error}"))
    })?;
    let mut trailing = [0_u8; 1];
    if file.read(&mut trailing).map_err(|error| {
        BundleIoError::Io(format!("cannot verify bounded {context} length: {error}"))
    })? != 0
    {
        return Err(BundleIoError::Conflict(format!(
            "{context} grew during its bounded read"
        )));
    }
    let finished = file.metadata().map_err(|error| {
        BundleIoError::Io(format!("cannot re-inspect opened {context}: {error}"))
    })?;
    if FileIdentity::from_metadata(&finished) != expected {
        return Err(BundleIoError::Conflict(format!(
            "{context} changed during its bounded read"
        )));
    }
    Ok(Some((expected, bytes)))
}

fn write_synced_staging(
    parent: &Dir,
    staging: &OsStr,
    bytes: &[u8],
) -> Result<FileIdentity, BundleIoError> {
    let mut options = CapOpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent.open_with(staging, &options).map_err(|error| {
        BundleIoError::Io(format!(
            "cannot create sibling bundle staging file: {error}"
        ))
    })?;
    if let Err(error) = file
        .write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = parent.remove_file(staging);
        return Err(BundleIoError::Io(format!(
            "cannot flush and sync sibling bundle staging file: {error}"
        )));
    }
    let metadata = file.metadata().map_err(|error| {
        BundleIoError::Io(format!(
            "cannot inspect synced bundle staging file: {error}"
        ))
    })?;
    let identity = FileIdentity::from_metadata(&metadata);
    let expected_length = u64::try_from(bytes.len())
        .map_err(|_| BundleIoError::InvalidInput("staging length does not fit u64".to_owned()))?;
    if identity.length != expected_length {
        return Err(BundleIoError::Conflict(
            "synced bundle staging length changed unexpectedly".to_owned(),
        ));
    }
    Ok(identity)
}

fn publish_staged(
    pinned: &PinnedDestination,
    names: &TransactionNames,
    staged: FileIdentity,
    expected: Option<FileIdentity>,
    backup: Option<&OsStr>,
) -> Result<BundlePublishOutcome, BundleIoError> {
    pinned.verify_binding()?;
    if regular_identity(&pinned.parent, &pinned.name, "bundle destination")? != expected {
        return Err(BundleIoError::Conflict(
            "bundle destination changed before activation".to_owned(),
        ));
    }
    match (expected, backup) {
        (None, None) => activate_new(pinned, names, staged),
        (Some(prior), Some(backup)) => replace_existing(pinned, names, staged, prior, backup),
        _ => Err(BundleIoError::InvalidInput(
            "bundle replacement state is internally inconsistent".to_owned(),
        )),
    }
}

fn activate_new(
    pinned: &PinnedDestination,
    names: &TransactionNames,
    staged: FileIdentity,
) -> Result<BundlePublishOutcome, BundleIoError> {
    if let Some(error) = injected_activation_failure() {
        return Err(error);
    }
    pinned
        .parent
        .hard_link(&names.staging, &pinned.parent, &pinned.name)
        .map_err(|error| {
            BundleIoError::Conflict(format!("cannot atomically activate new bundle: {error}"))
        })?;
    if regular_identity(&pinned.parent, &pinned.name, "activated bundle")? != Some(staged) {
        return Err(BundleIoError::Conflict(
            "activated bundle identity differs from sibling staging".to_owned(),
        ));
    }
    if let Err(error) = pinned.verify_binding() {
        return rollback_new(&pinned.parent, &pinned.name, staged, &error.to_string());
    }
    if let Err(error) = sync_activated(&pinned.parent, &pinned.name) {
        return rollback_new(&pinned.parent, &pinned.name, staged, &error.to_string());
    }
    if let Err(error) = pinned.parent.remove_file(&names.staging) {
        return rollback_new(
            &pinned.parent,
            &pinned.name,
            staged,
            &format!("cannot remove published bundle staging link: {error}"),
        );
    }
    if let Err(error) = sync_directory(&pinned.parent) {
        return rollback_new(&pinned.parent, &pinned.name, staged, &error.to_string());
    }
    Ok(BundlePublishOutcome::Created)
}

fn replace_existing(
    pinned: &PinnedDestination,
    names: &TransactionNames,
    staged: FileIdentity,
    prior: FileIdentity,
    backup: &OsStr,
) -> Result<BundlePublishOutcome, BundleIoError> {
    pinned
        .parent
        .hard_link(&pinned.name, &pinned.parent, backup)
        .map_err(|error| {
            BundleIoError::Conflict(format!("cannot create bundle backup: {error}"))
        })?;
    if regular_identity(&pinned.parent, backup, "bundle backup")? != Some(prior) {
        return Err(BundleIoError::Conflict(
            "bundle backup identity differs from the prior destination".to_owned(),
        ));
    }
    pinned
        .parent
        .rename(&pinned.name, &pinned.parent, &names.prior_claim)
        .map_err(|error| {
            BundleIoError::Conflict(format!(
                "cannot claim prior bundle for replacement: {error}"
            ))
        })?;
    if regular_identity(&pinned.parent, &names.prior_claim, "claimed prior bundle")? != Some(prior)
    {
        return restore_unexpected_claim(pinned, names, backup, prior);
    }
    let activation = match injected_activation_failure() {
        Some(error) => Err(error),
        None => pinned
            .parent
            .hard_link(&names.staging, &pinned.parent, &pinned.name)
            .map_err(|error| BundleIoError::Io(format!("cannot activate replacement: {error}"))),
    };
    if let Err(error) = activation {
        return rollback_claimed(pinned, names, backup, prior, &error.to_string());
    }
    if regular_identity(&pinned.parent, &pinned.name, "activated replacement")? != Some(staged) {
        return rollback_activated(
            pinned,
            names,
            backup,
            prior,
            staged,
            "activated replacement identity changed",
        );
    }
    if let Err(error) = pinned.verify_binding() {
        return rollback_activated(pinned, names, backup, prior, staged, &error.to_string());
    }
    if let Err(error) = sync_activated(&pinned.parent, &pinned.name) {
        return rollback_activated(pinned, names, backup, prior, staged, &error.to_string());
    }
    if let Err(error) = cleanup_replacement(&pinned.parent, names) {
        return rollback_activated(pinned, names, backup, prior, staged, &error.to_string());
    }
    if regular_identity(&pinned.parent, backup, "retained bundle backup")? != Some(prior) {
        return Err(BundleIoError::Conflict(
            "retained bundle backup changed after activation".to_owned(),
        ));
    }
    if let Err(error) = pinned.verify_binding() {
        return rollback_activated(pinned, names, backup, prior, staged, &error.to_string());
    }
    if let Err(error) = sync_directory(&pinned.parent) {
        return rollback_activated(pinned, names, backup, prior, staged, &error.to_string());
    }
    Ok(BundlePublishOutcome::Replaced)
}

fn restore_unexpected_claim(
    pinned: &PinnedDestination,
    names: &TransactionNames,
    backup: &OsStr,
    prior: FileIdentity,
) -> Result<BundlePublishOutcome, BundleIoError> {
    if regular_identity(&pinned.parent, &pinned.name, "bundle destination")?.is_none() {
        let _ = pinned
            .parent
            .hard_link(&names.prior_claim, &pinned.parent, &pinned.name);
    }
    Err(BundleIoError::Conflict(format!(
        "bundle destination changed while claimed; prior bundle retained at {} with identity {:?}",
        backup.to_string_lossy(),
        prior.object
    )))
}

fn rollback_claimed(
    pinned: &PinnedDestination,
    names: &TransactionNames,
    backup: &OsStr,
    prior: FileIdentity,
    cause: &str,
) -> Result<BundlePublishOutcome, BundleIoError> {
    match regular_identity(&pinned.parent, &pinned.name, "rollback destination")? {
        None => pinned
            .parent
            .hard_link(backup, &pinned.parent, &pinned.name)
            .map_err(|error| {
                BundleIoError::Conflict(format!(
                    "{cause}; prior bundle retained at {} but rollback failed: {error}",
                    backup.to_string_lossy()
                ))
            })?,
        Some(identity) if identity == prior => {}
        Some(_) => {
            return Err(BundleIoError::Conflict(format!(
                "{cause}; rollback preserved a foreign destination and retained the prior bundle at {}",
                backup.to_string_lossy()
            )));
        }
    }
    if regular_identity(&pinned.parent, &pinned.name, "restored bundle")? != Some(prior) {
        return Err(BundleIoError::Conflict(format!(
            "{cause}; restored bundle identity is not the retained prior backup"
        )));
    }
    remove_optional(&pinned.parent, &names.prior_claim);
    sync_activated(&pinned.parent, &pinned.name)?;
    Err(BundleIoError::Io(cause.to_owned()))
}

fn rollback_activated(
    pinned: &PinnedDestination,
    names: &TransactionNames,
    backup: &OsStr,
    prior: FileIdentity,
    staged: FileIdentity,
    cause: &str,
) -> Result<BundlePublishOutcome, BundleIoError> {
    if regular_identity(&pinned.parent, &pinned.name, "rollback replacement")? != Some(staged) {
        return Err(BundleIoError::Conflict(format!(
            "{cause}; rollback preserved a foreign destination and retained the prior bundle at {}",
            backup.to_string_lossy()
        )));
    }
    pinned
        .parent
        .rename(&pinned.name, &pinned.parent, &names.failed_claim)
        .map_err(|error| {
            BundleIoError::Conflict(format!(
                "{cause}; cannot claim failed replacement for rollback: {error}"
            ))
        })?;
    if regular_identity(
        &pinned.parent,
        &names.failed_claim,
        "failed replacement claim",
    )? != Some(staged)
    {
        return Err(BundleIoError::Conflict(format!(
            "{cause}; failed replacement changed while claimed; prior retained at {}",
            backup.to_string_lossy()
        )));
    }
    pinned
        .parent
        .hard_link(backup, &pinned.parent, &pinned.name)
        .map_err(|error| {
            BundleIoError::Conflict(format!(
                "{cause}; prior retained at {} but rollback activation failed: {error}",
                backup.to_string_lossy()
            ))
        })?;
    if regular_identity(&pinned.parent, &pinned.name, "restored prior bundle")? != Some(prior) {
        return Err(BundleIoError::Conflict(format!(
            "{cause}; restored prior bundle identity changed"
        )));
    }
    remove_optional(&pinned.parent, &names.failed_claim);
    remove_optional(&pinned.parent, &names.prior_claim);
    remove_optional(&pinned.parent, &names.staging);
    sync_activated(&pinned.parent, &pinned.name)?;
    Err(BundleIoError::Io(cause.to_owned()))
}

fn rollback_new(
    parent: &Dir,
    destination: &OsStr,
    staged: FileIdentity,
    cause: &str,
) -> Result<BundlePublishOutcome, BundleIoError> {
    if regular_identity(parent, destination, "failed new bundle")? == Some(staged) {
        parent.remove_file(destination).map_err(|error| {
            BundleIoError::Conflict(format!(
                "{cause}; cannot remove failed new bundle activation: {error}"
            ))
        })?;
        sync_directory(parent)?;
        return Err(BundleIoError::Io(cause.to_owned()));
    }
    Err(BundleIoError::Conflict(format!(
        "{cause}; rollback preserved a foreign destination"
    )))
}

fn cleanup_replacement(parent: &Dir, names: &TransactionNames) -> Result<(), BundleIoError> {
    parent.remove_file(&names.staging).map_err(|error| {
        BundleIoError::Io(format!("cannot remove replacement staging link: {error}"))
    })?;
    parent.remove_file(&names.prior_claim).map_err(|error| {
        BundleIoError::Io(format!(
            "cannot remove prior bundle transaction claim: {error}"
        ))
    })?;
    Ok(())
}

fn sync_activated(parent: &Dir, destination: &OsStr) -> Result<(), BundleIoError> {
    if let Some(error) = injected_durability_failure() {
        return Err(error);
    }
    let mut options = CapOpenOptions::new();
    options.read(true).write(true).follow(FollowSymlinks::No);
    parent
        .open_with(destination, &options)
        .and_then(|file| file.sync_all())
        .map_err(|error| BundleIoError::Io(format!("cannot sync activated bundle: {error}")))?;
    sync_directory(parent)
}

#[cfg(unix)]
fn sync_directory(parent: &Dir) -> Result<(), BundleIoError> {
    let mut options = CapOpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    parent
        .open_with(".", &options)
        .map_err(|error| BundleIoError::Io(format!("cannot open bundle directory: {error}")))?
        .sync_all()
        .map_err(|error| BundleIoError::Io(format!("cannot sync bundle directory: {error}")))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn sync_directory(_parent: &Dir) -> Result<(), BundleIoError> {
    Ok(())
}

fn remove_if_identity(parent: &Dir, name: &OsStr, expected: FileIdentity) {
    if regular_identity(parent, name, "bundle transaction path")
        .ok()
        .flatten()
        == Some(expected)
    {
        let _ = parent.remove_file(name);
    }
}

fn remove_optional(parent: &Dir, name: &OsStr) {
    let _ = parent.remove_file(name);
}

#[cfg(test)]
thread_local! {
    static INJECT_ACTIVATION_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static INJECT_DURABILITY_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn injected_activation_failure() -> Option<BundleIoError> {
    INJECT_ACTIVATION_FAILURE.with(|injected| {
        injected
            .replace(false)
            .then(|| BundleIoError::Io("injected bundle activation failure".to_owned()))
    })
}

#[cfg(test)]
fn injected_durability_failure() -> Option<BundleIoError> {
    INJECT_DURABILITY_FAILURE.with(|injected| {
        injected
            .replace(false)
            .then(|| BundleIoError::Io("injected bundle durability failure".to_owned()))
    })
}

#[cfg(not(test))]
fn injected_activation_failure() -> Option<BundleIoError> {
    None
}

#[cfg(not(test))]
fn injected_durability_failure() -> Option<BundleIoError> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portable::{
        BundleEntryInput, BundleScope, BundleSourceIdentity, BundleSourceKind,
        PortableEntryClassification,
    };
    use std::fs;
    use tempfile::tempdir;

    fn request(body: &[u8]) -> BundleRequest {
        BundleRequest {
            source: BundleSourceIdentity {
                kind: BundleSourceKind::Project,
                id: "project-alpha".to_owned(),
                logical_digest: format!("sha256:{}", "a".repeat(64)),
            },
            scope: BundleScope::Project {
                id: "project-alpha".to_owned(),
            },
            entries: vec![BundleEntryInput {
                relative_path: ".hive/knowledge/Wiki/project.md".to_owned(),
                bytes: body.to_vec(),
                classification: PortableEntryClassification::CanonicalMarkdown,
            }],
        }
    }

    #[test]
    fn create_load_and_exact_byte_noop_are_durable() {
        let temporary = tempdir().expect("tempdir");
        let destination = temporary.path().join("knowledge.hivekb");
        let first = encode_and_publish_bundle(
            &request(b"alpha\n"),
            BundleLimits::default(),
            &destination,
            &BundlePublishMode::CreateOnly,
        )
        .expect("create");
        assert_eq!(first.outcome(), BundlePublishOutcome::Created);
        assert_eq!(first.backup_path(), None);
        let original = fs::read(&destination).expect("published bytes");
        let second = encode_and_publish_bundle(
            &request(b"alpha\n"),
            BundleLimits::default(),
            &destination,
            &BundlePublishMode::CreateOnly,
        )
        .expect("no-op");
        assert_eq!(second.outcome(), BundlePublishOutcome::Unchanged);
        assert_eq!(fs::read(&destination).expect("unchanged bytes"), original);
        let plan = load_bundle(&destination, BundleLimits::default()).expect("load");
        assert_eq!(plan.entries()[0].bytes(), b"alpha\n");
        assert_eq!(plan.archive_sha256(), first.archive_sha256());
    }

    #[test]
    fn different_existing_bytes_require_typed_replace_and_retained_backup() {
        let temporary = tempdir().expect("tempdir");
        let destination = temporary.path().join("knowledge.hivekb");
        encode_and_publish_bundle(
            &request(b"prior\n"),
            BundleLimits::default(),
            &destination,
            &BundlePublishMode::CreateOnly,
        )
        .expect("prior");
        let prior = fs::read(&destination).expect("prior bytes");
        assert!(matches!(
            encode_and_publish_bundle(
                &request(b"next\n"),
                BundleLimits::default(),
                &destination,
                &BundlePublishMode::CreateOnly,
            ),
            Err(BundleIoError::Conflict(message)) if message.contains("replace mode")
        ));
        assert_eq!(fs::read(&destination).expect("preserved prior"), prior);

        let mode = BundlePublishMode::Replace {
            backup_file_name: OsString::from("knowledge.prior.hivekb"),
        };
        let receipt = encode_and_publish_bundle(
            &request(b"next\n"),
            BundleLimits::default(),
            &destination,
            &mode,
        )
        .expect("replace");
        assert_eq!(receipt.outcome(), BundlePublishOutcome::Replaced);
        let backup = receipt.backup_path().expect("backup path");
        assert_eq!(fs::read(backup).expect("backup bytes"), prior);
        assert_eq!(
            load_bundle(&destination, BundleLimits::default())
                .expect("replacement")
                .entries()[0]
                .bytes(),
            b"next\n"
        );
    }

    #[test]
    fn injected_replacement_failure_restores_prior_and_retains_backup() {
        let temporary = tempdir().expect("tempdir");
        let destination = temporary.path().join("knowledge.hivekb");
        encode_and_publish_bundle(
            &request(b"prior\n"),
            BundleLimits::default(),
            &destination,
            &BundlePublishMode::CreateOnly,
        )
        .expect("prior");
        let prior = fs::read(&destination).expect("prior bytes");
        INJECT_ACTIVATION_FAILURE.with(|injected| injected.set(true));
        let mode = BundlePublishMode::Replace {
            backup_file_name: OsString::from("knowledge.rollback.hivekb"),
        };
        assert!(matches!(
            encode_and_publish_bundle(
                &request(b"next\n"),
                BundleLimits::default(),
                &destination,
                &mode,
            ),
            Err(BundleIoError::Io(message)) if message.contains("injected")
        ));
        assert_eq!(fs::read(&destination).expect("restored prior"), prior);
        assert_eq!(
            fs::read(temporary.path().join("knowledge.rollback.hivekb")).expect("retained backup"),
            prior
        );

        INJECT_DURABILITY_FAILURE.with(|injected| injected.set(true));
        let durability_mode = BundlePublishMode::Replace {
            backup_file_name: OsString::from("knowledge.durability.hivekb"),
        };
        assert!(matches!(
            encode_and_publish_bundle(
                &request(b"next\n"),
                BundleLimits::default(),
                &destination,
                &durability_mode,
            ),
            Err(BundleIoError::Io(message)) if message.contains("durability")
        ));
        assert_eq!(fs::read(&destination).expect("durability rollback"), prior);
        assert_eq!(
            fs::read(temporary.path().join("knowledge.durability.hivekb"))
                .expect("durability backup"),
            prior
        );

        let new_destination = temporary.path().join("new.hivekb");
        INJECT_DURABILITY_FAILURE.with(|injected| injected.set(true));
        assert!(matches!(
            encode_and_publish_bundle(
                &request(b"new\n"),
                BundleLimits::default(),
                &new_destination,
                &BundlePublishMode::CreateOnly,
            ),
            Err(BundleIoError::Io(message)) if message.contains("durability")
        ));
        assert!(!new_destination.exists());
        let names = fs::read_dir(temporary.path())
            .expect("directory")
            .map(|entry| entry.expect("entry").file_name())
            .collect::<Vec<_>>();
        assert!(!names
            .iter()
            .any(|name| name.to_string_lossy().starts_with(".hivekb-")));
    }

    #[test]
    fn occupied_or_unsafe_backup_never_gets_overwritten() {
        let temporary = tempdir().expect("tempdir");
        let destination = temporary.path().join("knowledge.hivekb");
        encode_and_publish_bundle(
            &request(b"prior\n"),
            BundleLimits::default(),
            &destination,
            &BundlePublishMode::CreateOnly,
        )
        .expect("prior");
        let backup = temporary.path().join("occupied.bak");
        fs::write(&backup, b"foreign").expect("foreign backup");
        let occupied = BundlePublishMode::Replace {
            backup_file_name: OsString::from("occupied.bak"),
        };
        assert!(encode_and_publish_bundle(
            &request(b"next\n"),
            BundleLimits::default(),
            &destination,
            &occupied,
        )
        .is_err());
        assert_eq!(fs::read(&backup).expect("foreign preserved"), b"foreign");

        let unsafe_mode = BundlePublishMode::Replace {
            backup_file_name: OsString::from("../escape.bak"),
        };
        assert!(matches!(
            encode_and_publish_bundle(
                &request(b"next\n"),
                BundleLimits::default(),
                &destination,
                &unsafe_mode,
            ),
            Err(BundleIoError::InvalidInput(_))
        ));
    }

    #[test]
    fn load_rejects_non_regular_and_oversized_sources_before_decode() {
        let temporary = tempdir().expect("tempdir");
        let directory = temporary.path().join("directory.hivekb");
        fs::create_dir(&directory).expect("directory source");
        assert!(matches!(
            load_bundle(&directory, BundleLimits::default()),
            Err(BundleIoError::Conflict(_))
        ));

        let oversized = temporary.path().join("oversized.hivekb");
        fs::write(&oversized, [0_u8; 64]).expect("oversized source");
        let limits = BundleLimits {
            max_archive_bytes: 32,
            ..BundleLimits::default()
        };
        assert!(matches!(
            load_bundle(&oversized, limits),
            Err(BundleIoError::Portable(PortableError::BudgetExceeded(message)))
                if message.contains("before read")
        ));
    }

    #[test]
    fn invalid_extension_is_rejected_without_writes() {
        let temporary = tempdir().expect("tempdir");
        let destination = temporary.path().join("knowledge.zip");
        assert!(matches!(
            encode_and_publish_bundle(
                &request(b"alpha\n"),
                BundleLimits::default(),
                &destination,
                &BundlePublishMode::CreateOnly,
            ),
            Err(BundleIoError::InvalidInput(_))
        ));
        assert!(!destination.exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_destination_and_parent_are_rejected_no_follow() {
        use std::os::unix::fs::symlink;

        let temporary = tempdir().expect("tempdir");
        let outside = temporary.path().join("outside.hivekb");
        fs::write(&outside, b"outside").expect("outside");
        let destination = temporary.path().join("linked.hivekb");
        symlink(&outside, &destination).expect("file symlink");
        assert!(matches!(
            load_bundle(&destination, BundleLimits::default()),
            Err(BundleIoError::Conflict(_))
        ));
        assert_eq!(fs::read(&outside).expect("outside preserved"), b"outside");

        let real_parent = temporary.path().join("real-parent");
        fs::create_dir(&real_parent).expect("real parent");
        let linked_parent = temporary.path().join("linked-parent");
        symlink(&real_parent, &linked_parent).expect("parent symlink");
        let through_link = linked_parent.join("knowledge.hivekb");
        assert!(matches!(
            encode_and_publish_bundle(
                &request(b"alpha\n"),
                BundleLimits::default(),
                &through_link,
                &BundlePublishMode::CreateOnly,
            ),
            Err(BundleIoError::Conflict(_))
        ));
        assert!(!real_parent.join("knowledge.hivekb").exists());
    }
}
