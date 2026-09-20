//! Internal activation implementation.

use super::{
    ambient_authority, digest_tree, fs, io, io_internal, is_exact_projection_path,
    known_hook_descriptor_paths, ownership_manifest, read_installed_answers, read_target_optional,
    validate_exact_projection_relative, validate_installed_against, validate_managed_relative,
    validate_project_relative, validate_projection_ownership, BTreeMap, BTreeSet, CapFsMetadataExt,
    CapabilityResolution, Dir, DirExt, ExactProjectionMutation, FollowSymlinks, MutationOutcome,
    OpenOptions, OpenOptionsFollowExt, Ordering, OsStr, OsString, Path, PathBuf,
    ProjectionCleanupFault, ProjectionExpectedBefore, Read, RenderError, SetupAnswers, SystemTime,
    Write, ACTIVATION_TEMP_COUNTER, UNIX_EPOCH,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn activate_staged(
    target: &Path,
    target_dir: &Dir,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    verify_ambient_target: bool,
    post_apply: Option<&dyn Fn() -> Result<(), RenderError>>,
) -> Result<(), RenderError> {
    activate_staged_impl_with_pin(
        target,
        target_dir,
        files,
        deletions,
        answers,
        resolution,
        projection_expected_before,
        activation_fault_from_environment(),
        None,
        None,
        post_apply,
        None,
        verify_ambient_target,
    )
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ActivationFault {
    #[cfg(any(debug_assertions, test))]
    pub(super) fail_after_operations: usize,
    #[cfg(any(debug_assertions, test))]
    pub(super) fail_rollback: bool,
    pub(super) projection_cleanup: Option<ProjectionCleanupFault>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ReplacePolicy {
    pub(super) destination_requires_backup: bool,
    pub(super) fail_after_backup: bool,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(super) fn activate_staged_impl_with_pin(
    target: &Path,
    target_dir: &Dir,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    fault: Option<ActivationFault>,
    after_target_open: Option<&dyn Fn()>,
    before_projection_claim: Option<&dyn Fn(&Path)>,
    post_apply: Option<&dyn Fn() -> Result<(), RenderError>>,
    before_rollback: Option<&dyn Fn()>,
    verify_ambient_target: bool,
) -> Result<(), RenderError> {
    stage_and_validate(
        verify_ambient_target.then_some(target),
        files,
        answers,
        resolution,
        staging_corruption_from_environment(),
    )?;

    if verify_ambient_target {
        verify_target_capability_current(target, target_dir)?;
    }
    if let Some(barrier) = after_target_open {
        barrier();
    }

    let mut previous = BTreeMap::<PathBuf, Option<Vec<u8>>>::new();
    let mut created_directories = Vec::new();
    let operation_paths: BTreeSet<_> = files.keys().chain(deletions.iter()).cloned().collect();
    for relative in &operation_paths {
        previous.insert(
            relative.clone(),
            read_capability_optional(target_dir, relative)?,
        );
    }
    for relative in files.keys() {
        if let Err(error) = capability_parent(target_dir, relative, true, &mut created_directories)
        {
            return activation_failed(
                &error,
                target_dir,
                &previous,
                files,
                &[],
                &created_directories,
                projection_expected_before,
                before_rollback,
                fault,
            );
        }
    }

    let mut applied = Vec::new();
    let mut operation_count = 0;
    for (relative, bytes) in files {
        if should_inject_activation_failure(fault, operation_count) {
            return activation_failed(
                &RenderError::Internal("injected activation I/O failure".to_owned()),
                target_dir,
                &previous,
                files,
                &applied,
                &created_directories,
                projection_expected_before,
                before_rollback,
                fault,
            );
        }
        let result = match projection_expected_before.get(relative) {
            Some(ProjectionExpectedBefore::Absent) => verify_projection_expected_before(
                target_dir,
                relative,
                &ProjectionExpectedBefore::Absent,
            )
            .and_then(|()| {
                create_capability_file_exclusive(target_dir, relative, bytes)
                    .map_err(|error| RenderError::Conflict(error.to_string()))
            })
            .map(|()| MutationOutcome::Applied),
            Some(ProjectionExpectedBefore::Exact(expected)) if expected == bytes => {
                verify_projection_expected_before(
                    target_dir,
                    relative,
                    &ProjectionExpectedBefore::Exact(expected.clone()),
                )
                .map(|()| MutationOutcome::Unchanged)
            }
            Some(ProjectionExpectedBefore::Exact(expected)) => mutate_exact_projection_claimed(
                target_dir,
                relative,
                expected,
                ExactProjectionMutation::Replace(bytes),
                before_projection_claim,
                fault.and_then(|value| value.projection_cleanup),
            ),
            None => replace_capability_file(target_dir, relative, bytes)
                .map_err(|error| RenderError::Internal(error.to_string()))
                .map(|()| MutationOutcome::Applied),
        };
        match result {
            Ok(MutationOutcome::Applied) => applied.push(relative.clone()),
            Ok(MutationOutcome::Unchanged) => {}
            Ok(MutationOutcome::AppliedWithCleanupError(error)) => {
                applied.push(relative.clone());
                return activation_failed(
                    &error,
                    target_dir,
                    &previous,
                    files,
                    &applied,
                    &created_directories,
                    projection_expected_before,
                    before_rollback,
                    fault,
                );
            }
            Err(error) => {
                return activation_failed(
                    &error,
                    target_dir,
                    &previous,
                    files,
                    &applied,
                    &created_directories,
                    projection_expected_before,
                    before_rollback,
                    fault,
                );
            }
        }
        operation_count += 1;
    }
    for relative in deletions {
        if should_inject_activation_failure(fault, operation_count) {
            return activation_failed(
                &RenderError::Internal("injected activation I/O failure".to_owned()),
                target_dir,
                &previous,
                files,
                &applied,
                &created_directories,
                projection_expected_before,
                before_rollback,
                fault,
            );
        }
        if let Some(ProjectionExpectedBefore::Exact(expected)) =
            projection_expected_before.get(relative)
        {
            match mutate_exact_projection_claimed(
                target_dir,
                relative,
                expected,
                ExactProjectionMutation::Delete,
                before_projection_claim,
                fault.and_then(|value| value.projection_cleanup),
            ) {
                Ok(MutationOutcome::Applied) => applied.push(relative.clone()),
                Ok(MutationOutcome::AppliedWithCleanupError(error)) => {
                    applied.push(relative.clone());
                    return activation_failed(
                        &error,
                        target_dir,
                        &previous,
                        files,
                        &applied,
                        &created_directories,
                        projection_expected_before,
                        before_rollback,
                        fault,
                    );
                }
                Ok(MutationOutcome::Unchanged) => {
                    return activation_failed(
                        &RenderError::Internal(
                            "projection deletion unexpectedly reported no live change".to_owned(),
                        ),
                        target_dir,
                        &previous,
                        files,
                        &applied,
                        &created_directories,
                        projection_expected_before,
                        before_rollback,
                        fault,
                    );
                }
                Err(error) => {
                    return activation_failed(
                        &error,
                        target_dir,
                        &previous,
                        files,
                        &applied,
                        &created_directories,
                        projection_expected_before,
                        before_rollback,
                        fault,
                    );
                }
            }
        } else if previous.get(relative).is_some_and(Option::is_some) {
            if let Err(error) = remove_capability_file(target_dir, relative) {
                return activation_failed(
                    &RenderError::Internal(format!("activation deletion failed: {error}")),
                    target_dir,
                    &previous,
                    files,
                    &applied,
                    &created_directories,
                    projection_expected_before,
                    before_rollback,
                    fault,
                );
            }
            applied.push(relative.clone());
        }
        operation_count += 1;
    }
    let validation = validate_capability_activation(target_dir, files, deletions, answers)
        .and_then(|()| {
            if verify_ambient_target {
                verify_target_capability_current(target, target_dir)
            } else {
                Ok(())
            }
        })
        .and_then(|()| match post_apply {
            Some(commit) => commit(),
            None => Ok(()),
        });
    if let Err(error) = validation {
        return activation_failed(
            &error,
            target_dir,
            &previous,
            files,
            &applied,
            &created_directories,
            projection_expected_before,
            before_rollback,
            fault,
        );
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(super) fn activate_staged_impl(
    target: &Path,
    target_dir: &Dir,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    fault: Option<ActivationFault>,
    after_target_open: Option<&dyn Fn()>,
    before_projection_claim: Option<&dyn Fn(&Path)>,
    before_rollback: Option<&dyn Fn()>,
) -> Result<(), RenderError> {
    activate_staged_impl_with_pin(
        target,
        target_dir,
        files,
        deletions,
        answers,
        resolution,
        projection_expected_before,
        fault,
        after_target_open,
        before_projection_claim,
        None,
        before_rollback,
        true,
    )
}

pub(super) fn stage_and_validate(
    target: Option<&Path>,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
    corrupt_after_render: bool,
) -> Result<(), RenderError> {
    let mut builder = tempfile::Builder::new();
    builder.prefix(".aigent-hive-stage-");
    let staging = match target {
        Some(target) => {
            let parent = target
                .parent()
                .ok_or_else(|| RenderError::Input("target has no parent directory".to_owned()))?;
            builder.tempdir_in(parent).map_err(io_internal)?
        }
        None => builder.tempdir().map_err(io_internal)?,
    };
    let staging_root = staging.path().canonicalize().map_err(io_internal)?;
    for (relative, bytes) in files {
        let staged = staging_root.join(relative);
        if let Some(directory) = staged.parent() {
            fs::create_dir_all(directory).map_err(io_internal)?;
        }
        fs::write(&staged, bytes).map_err(io_internal)?;
    }
    if corrupt_after_render {
        fs::write(
            staging_root.join(".hive/config/harness.toml"),
            b"injected invalid staged bytes\n",
        )
        .map_err(io_internal)?;
    }
    validate_staged(&staging_root, files, answers, resolution)
}

pub(super) fn validate_staged(
    staging: &Path,
    files: &BTreeMap<PathBuf, Vec<u8>>,
    answers: &SetupAnswers,
    resolution: &CapabilityResolution,
) -> Result<(), RenderError> {
    for (relative, expected) in files {
        let staged = staging.join(relative);
        if !staged.is_file() {
            return Err(RenderError::Verification(format!(
                "staged output is missing: {}",
                relative.display()
            )));
        }
        if fs::read(&staged).map_err(io_internal)? != *expected {
            return Err(RenderError::Verification(format!(
                "staged output bytes changed after render: {}",
                relative.display()
            )));
        }
    }
    let harness =
        fs::read_to_string(staging.join(".hive/config/harness.toml")).map_err(io_internal)?;
    let _: toml::Value = toml::from_str(&harness)
        .map_err(|error| RenderError::Verification(format!("invalid harness TOML: {error}")))?;
    validate_installed_against(staging, answers, resolution, files)
}

pub(super) fn open_target_capability(target: &Path) -> Result<Dir, RenderError> {
    let parent = target
        .parent()
        .ok_or_else(|| RenderError::Input("activation target has no parent".to_owned()))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let name = target
        .file_name()
        .ok_or_else(|| RenderError::Input("activation target has no directory name".to_owned()))?;
    let parent_dir = Dir::open_ambient_dir(parent, ambient_authority()).map_err(io_internal)?;
    parent_dir.open_dir_nofollow(name).map_err(|error| {
        RenderError::Conflict(format!(
            "activation target cannot be opened as a no-follow directory {}: {error}",
            target.display()
        ))
    })
}

pub(super) fn verify_target_capability_current(
    target: &Path,
    pinned: &Dir,
) -> Result<(), RenderError> {
    let current = open_target_capability(target)?;
    let pinned_metadata = pinned.dir_metadata().map_err(io_internal)?;
    let current_metadata = current.dir_metadata().map_err(io_internal)?;
    if CapFsMetadataExt::dev(&pinned_metadata) != CapFsMetadataExt::dev(&current_metadata)
        || CapFsMetadataExt::ino(&pinned_metadata) != CapFsMetadataExt::ino(&current_metadata)
    {
        return Err(RenderError::Conflict(format!(
            "activation target no longer resolves to the pinned directory: {}",
            target.display()
        )));
    }
    Ok(())
}

pub(super) fn capability_parent(
    target: &Dir,
    relative: &Path,
    create_missing: bool,
    created_directories: &mut Vec<PathBuf>,
) -> Result<Option<(Dir, OsString)>, RenderError> {
    validate_managed_relative(relative)?;
    capability_parent_validated(target, relative, create_missing, created_directories)
}

pub(super) fn capability_parent_validated(
    target: &Dir,
    relative: &Path,
    create_missing: bool,
    created_directories: &mut Vec<PathBuf>,
) -> Result<Option<(Dir, OsString)>, RenderError> {
    let file_name = relative
        .file_name()
        .ok_or_else(|| RenderError::Internal("managed file has no name".to_owned()))?
        .to_os_string();
    let mut current = target.try_clone().map_err(io_internal)?;
    let mut current_relative = PathBuf::new();
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let component = component.as_os_str();
            current_relative.push(component);
            match current.symlink_metadata(component) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    return Err(RenderError::Conflict(format!(
                        "managed path ancestor is not a no-follow directory: {}",
                        current_relative.display()
                    )));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound && !create_missing => {
                    return Ok(None);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    current.create_dir(component).map_err(|error| {
                        RenderError::Internal(format!(
                            "activation directory creation failed at {}: {error}",
                            current_relative.display()
                        ))
                    })?;
                    created_directories.push(current_relative.clone());
                }
                Err(error) => {
                    return Err(RenderError::Conflict(format!(
                        "managed path ancestor cannot be inspected at {}: {error}",
                        current_relative.display()
                    )));
                }
            }
            current = current.open_dir_nofollow(component).map_err(|error| {
                RenderError::Conflict(format!(
                    "managed path ancestor cannot be opened no-follow at {}: {error}",
                    current_relative.display()
                ))
            })?;
        }
    }
    Ok(Some((current, file_name)))
}

pub(super) fn open_capability_file_nofollow(
    parent: &Dir,
    file_name: &OsStr,
) -> io::Result<cap_std::fs::File> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    parent.open_with(file_name, &options)
}

pub(super) fn read_capability_optional(
    target: &Dir,
    relative: &Path,
) -> Result<Option<Vec<u8>>, RenderError> {
    let mut created = Vec::new();
    let Some((parent, file_name)) = capability_parent(target, relative, false, &mut created)?
    else {
        return Ok(None);
    };
    match parent.symlink_metadata(&file_name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_internal(error)),
        Ok(metadata) if metadata.is_file() => {
            let mut file =
                open_capability_file_nofollow(&parent, &file_name).map_err(io_internal)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(io_internal)?;
            Ok(Some(bytes))
        }
        Ok(_) => Err(RenderError::Conflict(format!(
            "managed file path is occupied by a non-file: {}",
            relative.display()
        ))),
    }
}

pub(super) fn verify_projection_expected_before(
    target: &Dir,
    relative: &Path,
    expected: &ProjectionExpectedBefore,
) -> Result<(), RenderError> {
    validate_exact_projection_relative(relative)
        .map_err(|error| RenderError::Conflict(error.to_string()))?;
    let current = read_capability_optional(target, relative)?;
    let matches = match (expected, current.as_deref()) {
        (ProjectionExpectedBefore::Absent, None) => true,
        (ProjectionExpectedBefore::Exact(expected), Some(current)) => current == expected,
        (ProjectionExpectedBefore::Absent, Some(_))
        | (ProjectionExpectedBefore::Exact(_), None) => false,
    };
    if matches {
        Ok(())
    } else {
        Err(RenderError::Conflict(format!(
            "Hive projection changed after ownership preflight: {}",
            relative.display()
        )))
    }
}

pub(super) struct ProjectionClaim {
    pub(super) parent: Dir,
    pub(super) quarantine: Option<Dir>,
    pub(super) quarantine_name: OsString,
    pub(super) destination_name: OsString,
    pub(super) recovery_path: PathBuf,
}

pub(super) fn mutate_exact_projection_claimed(
    target: &Dir,
    relative: &Path,
    expected: &[u8],
    mutation: ExactProjectionMutation<'_>,
    before_claim: Option<&dyn Fn(&Path)>,
    cleanup_fault: Option<ProjectionCleanupFault>,
) -> Result<MutationOutcome, RenderError> {
    verify_projection_expected_before(
        target,
        relative,
        &ProjectionExpectedBefore::Exact(expected.to_vec()),
    )?;
    if let Some(barrier) = before_claim {
        barrier(relative);
    }
    let claim = claim_projection_destination(target, relative)?;
    let claimed = read_claimed_projection(&claim)?;
    if claimed != expected {
        return Err(projection_claim_conflict(
            claim,
            relative,
            "claimed bytes differ from the exact ownership proof",
        ));
    }

    match mutation {
        ExactProjectionMutation::Replace(bytes) => publish_claimed_projection_replacement(
            claim,
            relative,
            bytes,
            cleanup_fault == Some(ProjectionCleanupFault::Replacement),
        ),
        ExactProjectionMutation::Delete => Ok(finish_claimed_projection_deletion(
            claim,
            relative,
            cleanup_fault == Some(ProjectionCleanupFault::Deletion),
        )),
    }
}

pub(super) fn claim_projection_destination(
    target: &Dir,
    relative: &Path,
) -> Result<ProjectionClaim, RenderError> {
    validate_exact_projection_relative(relative)
        .map_err(|error| RenderError::Conflict(error.to_string()))?;
    let mut created = Vec::new();
    let (parent, destination_name) = capability_parent(target, relative, false, &mut created)?
        .ok_or_else(|| {
            RenderError::Conflict(format!(
                "host Skill projection disappeared before it could be claimed: {}",
                relative.display()
            ))
        })?;
    let (quarantine, quarantine_name) = create_projection_quarantine(&parent).map_err(|error| {
        RenderError::Conflict(format!(
            "cannot allocate a private projection quarantine for {}: {error}",
            relative.display()
        ))
    })?;
    if let Err(error) = parent.rename(
        &destination_name,
        &quarantine,
        OsStr::new("claimed-SKILL.md"),
    ) {
        drop(quarantine);
        let _ = parent.remove_dir(&quarantine_name);
        return Err(RenderError::Conflict(format!(
            "host Skill projection changed before atomic claim at {}: {error}",
            relative.display()
        )));
    }
    let recovery_path = relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&quarantine_name)
        .join("claimed-SKILL.md");
    Ok(ProjectionClaim {
        parent,
        quarantine: Some(quarantine),
        quarantine_name,
        destination_name,
        recovery_path,
    })
}

pub(super) fn create_projection_quarantine(parent: &Dir) -> io::Result<(Dir, OsString)> {
    for _ in 0..128 {
        let counter = ACTIVATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let epoch_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let name = OsString::from(format!(
            ".aigent-hive-claim-{}-{epoch_nanos:x}-{counter:x}",
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
        "cannot allocate an exclusive projection quarantine",
    ))
}

pub(super) fn read_claimed_projection(claim: &ProjectionClaim) -> Result<Vec<u8>, RenderError> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle");
    let mut file = open_capability_file_nofollow(quarantine, OsStr::new("claimed-SKILL.md"))
        .map_err(|error| {
            RenderError::Conflict(format!(
                "claimed projection cannot be opened no-follow at {}: {error}",
                claim.recovery_path.display()
            ))
        })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|error| {
        RenderError::Conflict(format!(
            "claimed projection cannot be read at {}: {error}",
            claim.recovery_path.display()
        ))
    })?;
    Ok(bytes)
}

pub(super) fn projection_claim_conflict(
    claim: ProjectionClaim,
    relative: &Path,
    reason: &str,
) -> RenderError {
    let recovery_path = claim.recovery_path.clone();
    match restore_projection_claim(claim) {
        Ok(()) => RenderError::Conflict(format!(
            "{reason} at {}; racing bytes were restored without overwrite",
            relative.display()
        )),
        Err(error) => RenderError::Conflict(format!(
            "{reason} at {}; every claimed byte remains recoverable at {} because non-overwriting restore failed: {error}",
            relative.display(),
            recovery_path.display()
        )),
    }
}

pub(super) fn restore_projection_claim(mut claim: ProjectionClaim) -> io::Result<()> {
    let quarantine = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle");
    hive_core::file_ops::publish_exclusive(
        quarantine,
        OsStr::new("claimed-SKILL.md"),
        &claim.parent,
        &claim.destination_name,
    )?;
    quarantine.remove_file(OsStr::new("claimed-SKILL.md"))?;
    drop(claim.quarantine.take());
    claim.parent.remove_dir(&claim.quarantine_name)
}

pub(super) fn publish_claimed_projection_replacement(
    mut claim: ProjectionClaim,
    relative: &Path,
    bytes: &[u8],
    inject_cleanup_failure: bool,
) -> Result<MutationOutcome, RenderError> {
    let temporary_name = match create_capability_temporary(
        &claim.parent,
        ".aigent-hive-projection-publish",
        bytes,
    ) {
        Ok(name) => name,
        Err(error) => {
            return Err(projection_claim_conflict(
                claim,
                relative,
                &format!("cannot stage exact projection replacement: {error}"),
            ));
        }
    };
    if let Err(error) = hive_core::file_ops::publish_exclusive(
        &claim.parent,
        &temporary_name,
        &claim.parent,
        &claim.destination_name,
    ) {
        let _ = claim.parent.remove_file(&temporary_name);
        return Err(projection_claim_conflict(
            claim,
            relative,
            &format!("exclusive projection publication was blocked: {error}"),
        ));
    }
    if inject_cleanup_failure {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "injected projection replacement cleanup failure after publication at {}; prior exact bytes remain recoverable at {}",
                relative.display(),
                claim.recovery_path.display()
            )),
        ));
    }
    if let Err(error) = claim.parent.remove_file(&temporary_name) {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "projection replacement published at {}, but private temporary cleanup failed; prior exact bytes remain recoverable at {}: {error}",
                relative.display(),
                claim.recovery_path.display()
            )),
        ));
    }
    if let Err(error) = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle")
        .remove_file(OsStr::new("claimed-SKILL.md"))
    {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "projection replacement published at {}, while prior exact bytes remain recoverable at {} because quarantine cleanup failed: {error}",
                relative.display(),
                claim.recovery_path.display()
            )),
        ));
    }
    drop(claim.quarantine.take());
    if let Err(error) = claim.parent.remove_dir(&claim.quarantine_name) {
        return Ok(MutationOutcome::AppliedWithCleanupError(
            RenderError::Rollback(format!(
                "projection replacement published at {}, but its empty private quarantine could not be removed: {error}",
                relative.display()
            )),
        ));
    }
    Ok(MutationOutcome::Applied)
}

pub(super) fn finish_claimed_projection_deletion(
    mut claim: ProjectionClaim,
    relative: &Path,
    inject_cleanup_failure: bool,
) -> MutationOutcome {
    if inject_cleanup_failure {
        return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                "injected projection deletion cleanup failure after atomic claim at {}; prior exact bytes remain recoverable at {}",
                relative.display(),
                claim.recovery_path.display()
            )));
    }
    match claim.parent.symlink_metadata(&claim.destination_name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Ok(_) => {
            return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                    "concurrent destination appeared while deleting {}; prior exact bytes remain recoverable at {}",
                    relative.display(),
                    claim.recovery_path.display()
                )));
        }
        Err(error) => {
            return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                    "cannot verify exclusive deletion destination at {}; prior exact bytes remain recoverable at {}: {error}",
                    relative.display(),
                    claim.recovery_path.display()
                )));
        }
    }
    if let Err(error) = claim
        .quarantine
        .as_ref()
        .expect("live projection claim retains its quarantine handle")
        .remove_file(OsStr::new("claimed-SKILL.md"))
    {
        return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                "projection deletion could not finalize at {}; exact bytes remain recoverable at {}: {error}",
                relative.display(),
                claim.recovery_path.display()
            )));
    }
    drop(claim.quarantine.take());
    if let Err(error) = claim.parent.remove_dir(&claim.quarantine_name) {
        return MutationOutcome::AppliedWithCleanupError(RenderError::Rollback(format!(
                "projection deletion completed at {}, but its empty private quarantine could not be removed: {error}",
                relative.display()
            )));
    }
    MutationOutcome::Applied
}

pub(super) fn create_capability_file_exclusive(
    target: &Dir,
    destination: &Path,
    bytes: &[u8],
) -> io::Result<()> {
    let mut created = Vec::new();
    let (parent, file_name) = capability_parent(target, destination, false, &mut created)
        .map_err(|error| render_error_to_io(&error))?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "managed parent is missing"))?;
    match hive_core::file_ops::stage(&parent, &file_name, bytes, |_| Ok::<_, io::Error>(())) {
        Ok(()) => Ok(()),
        Err(hive_core::file_ops::StageError::Create(error)) => Err(error),
        Err(
            hive_core::file_ops::StageError::Configure(error)
            | hive_core::file_ops::StageError::Write(error),
        ) => {
            let _ = parent.remove_file(&file_name);
            Err(error)
        }
    }
}

pub(super) fn replace_capability_file(
    target: &Dir,
    destination: &Path,
    bytes: &[u8],
) -> io::Result<()> {
    replace_capability_file_impl(
        target,
        destination,
        bytes,
        ReplacePolicy {
            destination_requires_backup: cfg!(windows),
            fail_after_backup: false,
        },
    )
}

pub(super) fn replace_capability_file_impl(
    target: &Dir,
    destination: &Path,
    bytes: &[u8],
    policy: ReplacePolicy,
) -> io::Result<()> {
    let mut created = Vec::new();
    let (parent, file_name) = capability_parent(target, destination, false, &mut created)
        .map_err(|error| render_error_to_io(&error))?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "managed parent is missing"))?;
    let destination_exists = match parent.symlink_metadata(&file_name) {
        Ok(metadata) if metadata.is_file() => true,
        Ok(_) => {
            return Err(io::Error::other(format!(
                "managed destination is occupied by a non-file: {}",
                destination.display()
            )));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    let temporary_name = create_capability_temporary(&parent, ".aigent-hive-activate", bytes)?;
    if !destination_exists || !policy.destination_requires_backup {
        let result = parent.rename(&temporary_name, &parent, &file_name);
        if result.is_err() {
            let _ = parent.remove_file(&temporary_name);
        }
        return result;
    }

    let backup_name = create_capability_temporary(&parent, ".aigent-hive-replace-backup", b"")?;
    if let Err(error) = parent.rename(&file_name, &parent, &backup_name) {
        let _ = parent.remove_file(&temporary_name);
        let _ = parent.remove_file(&backup_name);
        return Err(error);
    }
    if policy.fail_after_backup {
        let error = io::Error::other("injected failure after destination backup");
        let restored =
            restore_capability_replacement_backup(&parent, &file_name, &backup_name, error);
        let _ = parent.remove_file(&temporary_name);
        return Err(restored);
    }
    if let Err(error) = parent.rename(&temporary_name, &parent, &file_name) {
        let restored =
            restore_capability_replacement_backup(&parent, &file_name, &backup_name, error);
        let _ = parent.remove_file(&temporary_name);
        return Err(restored);
    }
    match parent.remove_file(&backup_name) {
        Ok(()) => Ok(()),
        Err(error) => Err(restore_capability_replacement_backup(
            &parent,
            &file_name,
            &backup_name,
            error,
        )),
    }
}

pub(super) fn create_capability_temporary(
    parent: &Dir,
    prefix: &str,
    bytes: &[u8],
) -> io::Result<OsString> {
    for _ in 0..128 {
        let counter = ACTIVATION_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let epoch_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let name = OsString::from(format!(
            "{prefix}-{}-{epoch_nanos:x}-{counter:x}",
            std::process::id()
        ));
        match hive_core::file_ops::stage(parent, &name, bytes, |_| Ok::<_, io::Error>(())) {
            Ok(()) => return Ok(name),
            Err(hive_core::file_ops::StageError::Create(error))
                if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(hive_core::file_ops::StageError::Create(error)) => return Err(error),
            Err(
                hive_core::file_ops::StageError::Configure(error)
                | hive_core::file_ops::StageError::Write(error),
            ) => {
                let _ = parent.remove_file(&name);
                return Err(error);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "cannot allocate an exclusive activation temporary file",
    ))
}

pub(super) fn restore_capability_replacement_backup(
    parent: &Dir,
    destination: &OsStr,
    backup: &OsStr,
    original: io::Error,
) -> io::Error {
    match parent.remove_file(destination) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return io::Error::other(format!(
                "{original}; replacement cleanup failed before restore: {error}"
            ));
        }
    }
    match parent.rename(backup, parent, destination) {
        Ok(()) => original,
        Err(error) => io::Error::other(format!(
            "{original}; destination backup restore failed: {error}"
        )),
    }
}

pub(super) fn remove_capability_file(target: &Dir, relative: &Path) -> io::Result<()> {
    let mut created = Vec::new();
    let Some((parent, file_name)) = capability_parent(target, relative, false, &mut created)
        .map_err(|error| render_error_to_io(&error))?
    else {
        return Ok(());
    };
    match parent.remove_file(file_name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn remove_created_capability_directory(
    target: &Dir,
    relative: &Path,
    _projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
) -> io::Result<()> {
    let mut created = Vec::new();
    let Some((parent, directory_name)) =
        capability_parent_validated(target, relative, false, &mut created)
            .map_err(|error| render_error_to_io(&error))?
    else {
        return Ok(());
    };
    match parent.remove_dir(&directory_name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            let directory = parent.open_dir_nofollow(&directory_name)?;
            if directory.entries()?.next().is_some() {
                // A concurrent foreign entry owns the surviving directory. The
                // rollback has removed every Hive-created file, so preserving
                // the non-empty container is the safe, complete outcome.
                Ok(())
            } else {
                Err(error)
            }
        }
    }
}

pub(super) fn validate_capability_activation(
    target: &Dir,
    planned: &BTreeMap<PathBuf, Vec<u8>>,
    deletions: &BTreeSet<PathBuf>,
    answers: &SetupAnswers,
) -> Result<(), RenderError> {
    for (relative, expected) in planned {
        let current = read_capability_optional(target, relative)?.ok_or_else(|| {
            RenderError::Verification(format!(
                "activated managed output is missing: {}",
                relative.display()
            ))
        })?;
        if current != *expected {
            return Err(RenderError::Verification(format!(
                "activated managed output differs from staged bytes: {}",
                relative.display()
            )));
        }
    }
    for relative in deletions {
        if read_capability_optional(target, relative)?.is_some() {
            return Err(RenderError::Verification(format!(
                "activated deletion remains installed: {}",
                relative.display()
            )));
        }
    }
    let approved_hooks: BTreeSet<_> = answers
        .approved_fallback_hooks
        .iter()
        .map(|hook| PathBuf::from(&hook.path))
        .collect();
    for relative in known_hook_descriptor_paths() {
        if !approved_hooks.contains(&relative)
            && read_capability_optional(target, &relative)?.is_some()
        {
            return Err(RenderError::Verification(format!(
                "unapproved known fallback hook remains installed: {}",
                relative.display()
            )));
        }
    }
    let _ = capability_tree_digest(target)?;
    Ok(())
}

pub(super) fn capability_tree_digest(target: &Dir) -> Result<String, RenderError> {
    let manifest = ownership_manifest()?;
    let mut entries = BTreeMap::new();
    for entry in manifest.paths {
        if let Some(prefix) = entry.pattern.strip_suffix("/**") {
            collect_capability_owned_files(target, Path::new(prefix), &mut entries)?;
        } else if !entry.pattern.contains('*') {
            let relative = PathBuf::from(entry.pattern);
            if let Some(bytes) = read_capability_optional(target, &relative)? {
                entries.insert(relative, bytes);
            }
        }
    }
    if read_target_optional(target, Path::new(".hive/config/active-skills.yml"))?.is_some() {
        let answers = read_installed_answers(target)?;
        entries.extend(validate_projection_ownership(target, &answers)?.files);
    }
    Ok(digest_tree(&entries))
}

pub(super) fn collect_capability_owned_files(
    target: &Dir,
    relative: &Path,
    entries: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    validate_project_relative(relative).map_err(|error| RenderError::Safety(error.to_string()))?;
    let mut created = Vec::new();
    let Some((parent, name)) = capability_parent(target, relative, false, &mut created)? else {
        return Ok(());
    };
    let metadata = match parent.symlink_metadata(&name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_internal(error)),
    };
    if metadata.is_file() {
        let bytes = read_capability_optional(target, relative)?.ok_or_else(|| {
            RenderError::Internal(format!(
                "owned file disappeared during activation validation: {}",
                relative.display()
            ))
        })?;
        entries.insert(relative.to_path_buf(), bytes);
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(RenderError::Safety(format!(
            "owned tree contains a non-file, non-directory entry: {}",
            relative.display()
        )));
    }
    let directory = open_capability_child_directory(&parent, &name, relative)?;
    collect_capability_directory_entries(&directory, relative, entries)
}

pub(super) fn collect_capability_directory_entries(
    directory: &Dir,
    relative: &Path,
    entries: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), RenderError> {
    let mut children = directory
        .entries()
        .map_err(io_internal)?
        .map(|entry| entry.map(|entry| entry.file_name()).map_err(io_internal))
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();
    for child in children {
        let child_relative = relative.join(&child);
        validate_project_relative(&child_relative)
            .map_err(|error| RenderError::Safety(error.to_string()))?;
        let metadata = directory.symlink_metadata(&child).map_err(io_internal)?;
        if metadata.is_file() {
            let mut file = open_capability_file_nofollow(directory, &child).map_err(io_internal)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(io_internal)?;
            entries.insert(child_relative, bytes);
        } else if metadata.is_dir() {
            let child_directory =
                open_capability_child_directory(directory, &child, &child_relative)?;
            collect_capability_directory_entries(&child_directory, &child_relative, entries)?;
        } else {
            return Err(RenderError::Safety(format!(
                "owned tree contains a non-file, non-directory entry: {}",
                child_relative.display()
            )));
        }
    }
    Ok(())
}

pub(super) fn open_capability_child_directory(
    parent: &Dir,
    name: &OsStr,
    relative: &Path,
) -> Result<Dir, RenderError> {
    parent.open_dir_nofollow(name).map_err(|error| {
        RenderError::Safety(format!(
            "owned directory cannot be opened no-follow at {}: {error}",
            relative.display()
        ))
    })
}

pub(super) struct ProjectionRecovery {
    pub(super) parent: Dir,
    pub(super) quarantine: Option<Dir>,
    pub(super) quarantine_name: OsString,
    pub(super) destination_name: OsString,
    pub(super) recovery_path: PathBuf,
}

pub(super) fn stage_projection_recovery(
    target: &Dir,
    relative: &Path,
    bytes: &[u8],
) -> Result<ProjectionRecovery, RenderError> {
    validate_exact_projection_relative(relative)
        .map_err(|error| RenderError::Rollback(error.to_string()))?;
    let mut created = Vec::new();
    let (parent, destination_name) = capability_parent(target, relative, false, &mut created)?
        .ok_or_else(|| {
            RenderError::Rollback(format!(
                "projection rollback parent is missing: {}",
                relative.display()
            ))
        })?;
    let (quarantine, quarantine_name) = create_projection_quarantine(&parent).map_err(|error| {
        RenderError::Rollback(format!(
            "cannot allocate projection rollback recovery for {}: {error}",
            relative.display()
        ))
    })?;
    let recovery_name = OsStr::new("prior-SKILL.md");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let mut file = match quarantine.open_with(recovery_name, &options) {
        Ok(file) => file,
        Err(error) => {
            drop(quarantine);
            let _ = parent.remove_dir(&quarantine_name);
            return Err(RenderError::Rollback(format!(
                "cannot create projection rollback recovery for {}: {error}",
                relative.display()
            )));
        }
    };
    if let Err(error) = file
        .write_all(bytes)
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = quarantine.remove_file(recovery_name);
        drop(quarantine);
        let _ = parent.remove_dir(&quarantine_name);
        return Err(RenderError::Rollback(format!(
            "cannot persist projection rollback recovery for {}: {error}",
            relative.display()
        )));
    }
    drop(file);
    let recovery_path = relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&quarantine_name)
        .join(recovery_name);
    Ok(ProjectionRecovery {
        parent,
        quarantine: Some(quarantine),
        quarantine_name,
        destination_name,
        recovery_path,
    })
}

pub(super) fn cleanup_projection_recovery(mut recovery: ProjectionRecovery) -> io::Result<()> {
    recovery
        .quarantine
        .as_ref()
        .expect("live projection recovery retains its quarantine handle")
        .remove_file(OsStr::new("prior-SKILL.md"))?;
    drop(recovery.quarantine.take());
    recovery.parent.remove_dir(&recovery.quarantine_name)
}

pub(super) fn rollback_projection(
    target: &Dir,
    relative: &Path,
    prior: Option<&[u8]>,
    published: Option<&[u8]>,
) -> Result<(), RenderError> {
    match (prior, published) {
        (Some(prior), Some(published)) => {
            rollback_replaced_projection(target, relative, prior, published)
        }
        (None, Some(published)) => rollback_created_projection(target, relative, published),
        (Some(prior), None) => rollback_deleted_projection(target, relative, prior),
        (None, None) => Err(RenderError::Rollback(format!(
            "{}: projection rollback has neither prior nor published bytes",
            relative.display()
        ))),
    }
}

pub(super) fn rollback_replaced_projection(
    target: &Dir,
    relative: &Path,
    prior: &[u8],
    published: &[u8],
) -> Result<(), RenderError> {
    let recovery = stage_projection_recovery(target, relative, prior)?;
    let recovery_path = recovery.recovery_path.clone();
    match mutate_exact_projection_claimed(
        target,
        relative,
        published,
        ExactProjectionMutation::Replace(prior),
        None,
        None,
    ) {
        Ok(MutationOutcome::Applied) => {
            cleanup_projection_recovery(recovery).map_err(|error| {
                RenderError::Rollback(format!(
                    "{}: prior bytes were restored, but private rollback recovery cleanup failed at {}: {error}",
                    relative.display(),
                    recovery_path.display()
                ))
            })
        }
        Ok(MutationOutcome::AppliedWithCleanupError(error)) => {
            let recovery_cleanup = cleanup_projection_recovery(recovery)
                .err()
                .map(|cleanup| format!("; rollback recovery cleanup also failed: {cleanup}"))
                .unwrap_or_default();
            Err(RenderError::Rollback(format!(
                "{}: prior bytes were restored, but reverse projection cleanup remains incomplete: {error}{recovery_cleanup}",
                relative.display()
            )))
        }
        Ok(MutationOutcome::Unchanged) => Err(RenderError::Rollback(format!(
            "{}: projection rollback unexpectedly reported no live change",
            relative.display()
        ))),
        Err(error) => {
            drop(recovery);
            Err(RenderError::Rollback(format!(
                "{}: live post-activation bytes changed before rollback; live/claimed bytes were preserved and prior bytes remain recoverable at {}: {error}",
                relative.display(),
                recovery_path.display()
            )))
        }
    }
}

pub(super) fn rollback_created_projection(
    target: &Dir,
    relative: &Path,
    published: &[u8],
) -> Result<(), RenderError> {
    mutate_exact_projection_claimed(
        target,
        relative,
        published,
        ExactProjectionMutation::Delete,
        None,
        None,
    )
    .and_then(|outcome| match outcome {
        MutationOutcome::Applied => Ok(()),
        MutationOutcome::AppliedWithCleanupError(error) => Err(RenderError::Rollback(format!(
            "{}: newly created projection was removed, but reverse cleanup remains incomplete: {error}",
            relative.display()
        ))),
        MutationOutcome::Unchanged => Err(RenderError::Rollback(format!(
            "{}: new projection rollback unexpectedly reported no live change",
            relative.display()
        ))),
    })
    .map_err(|error| match error {
        RenderError::Rollback(message) => RenderError::Rollback(message),
        other => RenderError::Rollback(format!(
            "{}: newly created projection changed before rollback and was preserved: {other}",
            relative.display()
        )),
    })
}

pub(super) fn rollback_deleted_projection(
    target: &Dir,
    relative: &Path,
    prior: &[u8],
) -> Result<(), RenderError> {
    let recovery = stage_projection_recovery(target, relative, prior)?;
    let recovery_path = recovery.recovery_path.clone();
    let quarantine = recovery
        .quarantine
        .as_ref()
        .expect("live projection recovery retains its quarantine handle");
    match hive_core::file_ops::publish_exclusive(quarantine,
        OsStr::new("prior-SKILL.md"),
        &recovery.parent,
        &recovery.destination_name,
    ) {
        Ok(()) => cleanup_projection_recovery(recovery).map_err(|error| {
            RenderError::Rollback(format!(
                "{}: deleted projection bytes were restored, but private rollback recovery cleanup failed at {}: {error}",
                relative.display(),
                recovery_path.display()
            ))
        }),
        Err(error) => {
            drop(recovery);
            Err(RenderError::Rollback(format!(
                "{}: rollback refused to overwrite a newly occupied projection path; live bytes remain and prior bytes are recoverable at {}: {error}",
                relative.display(),
                recovery_path.display()
            )))
        }
    }
}

pub(super) fn rollback(
    target: &Dir,
    previous: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    planned: &BTreeMap<PathBuf, Vec<u8>>,
    applied: &[PathBuf],
    created_directories: &[PathBuf],
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
) -> Result<(), RenderError> {
    let mut errors = Vec::new();
    for relative in applied.iter().rev() {
        if is_exact_projection_path(relative) {
            let prior = previous.get(relative).ok_or_else(|| {
                RenderError::Rollback(format!(
                    "hive.activation-rollback-failed: {}: missing projection activation snapshot",
                    relative.display()
                ))
            })?;
            if let Err(error) = rollback_projection(
                target,
                relative,
                prior.as_deref(),
                planned.get(relative).map(Vec::as_slice),
            ) {
                errors.push(error.to_string());
            }
            continue;
        }
        match previous.get(relative) {
            Some(Some(content)) => {
                if let Err(error) = replace_capability_file(target, relative, content) {
                    errors.push(format!("{}: {error}", relative.display()));
                }
            }
            Some(None) => {
                if let Err(error) = remove_capability_file(target, relative) {
                    errors.push(format!("{}: {error}", relative.display()));
                }
            }
            None => {
                errors.push(format!(
                    "{}: missing activation snapshot",
                    relative.display()
                ));
            }
        }
    }
    for directory in created_directories.iter().rev() {
        if let Err(error) =
            remove_created_capability_directory(target, directory, projection_expected_before)
        {
            errors.push(format!("{}: {error}", directory.display()));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(RenderError::Rollback(format!(
            "hive.activation-rollback-failed: {}",
            errors.join("; ")
        )))
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn activation_failed(
    original: &RenderError,
    target: &Dir,
    previous: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    planned: &BTreeMap<PathBuf, Vec<u8>>,
    applied: &[PathBuf],
    created_directories: &[PathBuf],
    projection_expected_before: &BTreeMap<PathBuf, ProjectionExpectedBefore>,
    before_rollback: Option<&dyn Fn()>,
    fault: Option<ActivationFault>,
) -> Result<(), RenderError> {
    #[cfg(not(any(debug_assertions, test)))]
    let _ = fault;
    #[cfg(any(debug_assertions, test))]
    if fault.is_some_and(|value| value.fail_rollback) {
        return Err(RenderError::Rollback(
            "hive.activation-rollback-failed: injected rollback failure".to_owned(),
        ));
    }
    if let Some(barrier) = before_rollback {
        barrier();
    }
    rollback(
        target,
        previous,
        planned,
        applied,
        created_directories,
        projection_expected_before,
    )?;
    let message =
        format!("activation failed and was rolled back to the prior generation: {original}");
    match original {
        RenderError::Rollback(_) => Err(RenderError::Rollback(format!(
            "{original}; live paths were rolled back but projection cleanup evidence remains"
        ))),
        RenderError::Conflict(_) => Err(RenderError::Conflict(message)),
        _ => Err(RenderError::Internal(message)),
    }
}

pub(super) fn render_error_to_io(error: &RenderError) -> io::Error {
    io::Error::other(error.to_string())
}

pub(super) fn should_inject_activation_failure(
    fault: Option<ActivationFault>,
    count: usize,
) -> bool {
    #[cfg(any(debug_assertions, test))]
    {
        fault.is_some_and(|value| value.fail_after_operations == count)
    }
    #[cfg(not(any(debug_assertions, test)))]
    {
        let _ = (fault, count);
        false
    }
}

pub(super) fn activation_fault_from_environment() -> Option<ActivationFault> {
    #[cfg(debug_assertions)]
    {
        let value = std::env::var("HIVE_TEST_ACTIVATION_FAIL_AFTER").ok()?;
        let current_thread = format!("{:?}", std::thread::current().id());
        let fail_rollback =
            std::env::var_os("HIVE_TEST_ROLLBACK_FAIL").is_some_and(|value| value == "1");
        activation_fault_from_value(&value, &current_thread, fail_rollback)
    }
    #[cfg(not(debug_assertions))]
    {
        None
    }
}

#[cfg(any(debug_assertions, test))]
pub(super) fn activation_fault_from_value(
    value: &str,
    current_thread: &str,
    fail_rollback: bool,
) -> Option<ActivationFault> {
    let (scope, fail_after_operations) = value
        .rsplit_once('@')
        .map_or((None, value), |(scope, operations)| {
            (Some(scope), operations)
        });
    if scope.is_some_and(|scope| scope != current_thread) {
        return None;
    }
    let fail_after_operations = fail_after_operations
        .parse::<usize>()
        .ok()
        .filter(|value| *value <= 4096)?;
    Some(ActivationFault {
        fail_after_operations,
        fail_rollback,
        projection_cleanup: None,
    })
}

pub(super) fn staging_corruption_from_environment() -> bool {
    #[cfg(debug_assertions)]
    {
        std::env::var_os("HIVE_TEST_STAGING_CORRUPT_AFTER_RENDER").is_some_and(|value| value == "1")
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}
