//! A confirmed removal retains its exact inventory outside all deletion targets.
use super::super::{
    deactivation, emit_action_result, failure, file_permissions, inspect_setup_write_claim,
    lock_user_setup, parse_uninstall, portable, prune_empty_owned_ancestors, read_optional_regular,
    recover_user_setup_file, remove_hive_guidance_marker, remove_owned_empty_dir,
    remove_regular_if_exists, replace_user_setup_file, valid_sha256, validate_relative,
    write_atomic, InstallError, UserUninstallArguments, UNINSTALL_FILES, UNINSTALL_TREES,
};
use super::{digest, root_identity, tree, verify, Preview, PRESERVED};
use crate::{usage::SystemCommandRunner, ActionResult};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

const LIMIT: u64 = 4 * 1024 * 1024;

#[derive(Clone, Copy, Deserialize, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Phase {
    Approved,
    MonitorStopped,
    HostsRemoved,
    FilesRemoved,
    Complete,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u8,
    operation_id: String,
    product_version: String,
    preview_digest: String,
    plan: Preview,
    phase: Phase,
    changed_paths: Vec<String>,
    language: crate::user_setup::InterfaceLanguage,
    last_error: Option<String>,
}

fn path(id: &str) -> Result<PathBuf, InstallError> {
    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(InstallError::Input("invalid removal operation ID".into()));
    }
    Ok(Path::new(".hive/history/lifecycle").join(format!("uninstall-{id}.json")))
}

fn validate(target: &UserUninstallArguments, receipt: &Receipt) -> Result<(), InstallError> {
    path(&receipt.operation_id)?;
    if receipt.schema_version != 1
        || receipt.product_version != env!("CARGO_PKG_VERSION")
        || receipt.plan.root != target.user_root
        || receipt.plan.root_identity != root_identity(&target.root_cap)?
        || receipt.preview_digest != digest(&receipt.plan)?
        || receipt.plan.entries.len() > 16_384
    {
        return Err(InstallError::Conflict(
            "removal receipt no longer matches its approved root and inventory".into(),
        ));
    }
    for (name, entry) in &receipt.plan.entries {
        let relative = Path::new(name);
        validate_relative(relative)?;
        let allowed = UNINSTALL_TREES
            .iter()
            .any(|base| relative.starts_with(base))
            || UNINSTALL_FILES
                .iter()
                .any(|file| name == file && name != ".hive/config/projects.yml")
            || crate::user_setup::valid_user_projection_path(relative)
            || relative.strip_prefix(".gemini/config").is_ok_and(|skill| {
                crate::user_setup::valid_user_projection_path(Path::new(&format!(
                    ".agents/{}",
                    portable(skill)
                )))
            })
            || (entry.kind == "marker"
                && matches!(
                    name.as_str(),
                    ".codex/AGENTS.md"
                        | ".codex/AGENTS.override.md"
                        | ".claude/CLAUDE.md"
                        | ".gemini/GEMINI.md"
                ));
        if !allowed
            || !matches!(
                entry.kind.as_str(),
                "file" | "directory" | "marker" | "preserved"
            )
            || (entry.digest.is_none()
                && entry.kind != "directory"
                && !matches!(
                    name.as_str(),
                    ".hive/runtime/usage-monitor/state.json"
                        | ".hive/runtime/usage-monitor/startup.json"
                ))
            || entry
                .digest
                .as_ref()
                .is_some_and(|digest| !valid_sha256(digest))
            || entry
                .after_digest
                .as_ref()
                .is_some_and(|digest| !valid_sha256(digest))
        {
            return Err(InstallError::Conflict(format!(
                "removal receipt contains an unowned path or invalid fingerprint: {name}"
            )));
        }
    }
    Ok(())
}

fn save(target: &UserUninstallArguments, receipt: &Receipt) -> Result<(), InstallError> {
    validate(target, receipt)?;
    let path = path(&receipt.operation_id)?;
    let bytes = serde_json::to_vec(receipt).map_err(|e| InstallError::Internal(e.to_string()))?;
    if bytes.len() as u64 > LIMIT {
        return Err(InstallError::Conflict(
            "removal receipt exceeds its size limit".into(),
        ));
    }
    let previous = read_optional_regular(&target.root_cap, &path, LIMIT)?;
    replace_user_setup_file(&target.root_cap, &path, previous.as_deref(), Some(&bytes))
        .map_err(InstallError::Conflict)
}

fn read(target: &UserUninstallArguments, id: &str) -> Result<Receipt, InstallError> {
    let relative = path(id)?;
    // The existing CAS claim contains the complete pending replacement. Validate its
    // scope before allowing the same file writer to finish an interrupted publication.
    if let Some(claim) = inspect_setup_write_claim(&target.root_cap, &relative, LIMIT)
        .map_err(InstallError::Conflict)?
    {
        let candidate = claim
            .staged
            .as_ref()
            .or(claim.before.as_ref())
            .ok_or_else(|| InstallError::Conflict("removal receipt claim is empty".into()))?;
        let receipt: Receipt = serde_json::from_slice(candidate)
            .map_err(|_| InstallError::Conflict("invalid retained removal receipt".into()))?;
        validate(target, &receipt)?;
        if receipt.operation_id != id {
            return Err(InstallError::Conflict("removal receipt ID differs".into()));
        }
        recover_user_setup_file(
            &target.root_cap,
            &relative,
            &claim
                .before
                .as_deref()
                .map_or_else(|| "absent".into(), sha256_digest),
            &claim
                .staged
                .as_deref()
                .map_or_else(|| "absent".into(), sha256_digest),
            claim.before_mode,
        )
        .map_err(InstallError::Conflict)?;
    }
    let bytes = read_optional_regular(&target.root_cap, &relative, LIMIT)?
        .ok_or_else(|| InstallError::Conflict("approved removal receipt is missing".into()))?;
    let receipt: Receipt = serde_json::from_slice(&bytes)
        .map_err(|_| InstallError::Conflict("invalid removal receipt".into()))?;
    validate(target, &receipt)?;
    if receipt.operation_id != id {
        return Err(InstallError::Conflict("removal receipt ID differs".into()));
    }
    Ok(receipt)
}

pub(super) fn start(
    target: &UserUninstallArguments,
    plan: Preview,
    ko: bool,
) -> Result<ActionResult, InstallError> {
    let _lease = lock_user_setup(&target.root_cap).map_err(InstallError::Conflict)?;
    verify(
        target,
        &env::current_exe().map_err(|e| InstallError::Internal(e.to_string()))?,
        &plan,
    )?;
    let mut random = [0; 32];
    getrandom::fill(&mut random)
        .map_err(|_| InstallError::Internal("cannot identify removal".into()))?;
    let id = sha256_digest(&random)[7..].to_owned();
    let mut receipt = Receipt {
        schema_version: 1,
        operation_id: id,
        product_version: env!("CARGO_PKG_VERSION").into(),
        preview_digest: digest(&plan)?,
        plan,
        phase: Phase::Approved,
        changed_paths: Vec::new(),
        language: if ko {
            crate::user_setup::InterfaceLanguage::Ko
        } else {
            crate::user_setup::InterfaceLanguage::En
        },
        last_error: None,
    };
    save(target, &receipt)?;
    interrupt("approved");
    finish(target, &mut receipt)
}

pub(super) fn resume(
    target: &UserUninstallArguments,
    id: &str,
) -> Result<ActionResult, InstallError> {
    let _lease = lock_user_setup(&target.root_cap).map_err(InstallError::Conflict)?;
    let mut receipt = read(target, id)?;
    finish(target, &mut receipt)
}

fn finish(
    target: &UserUninstallArguments,
    receipt: &mut Receipt,
) -> Result<ActionResult, InstallError> {
    let mut result = apply(target, receipt);
    receipt.last_error = result.as_ref().err().map(|e| e.message().to_owned());
    save(target, receipt)?;
    if result.is_ok()
        && receipt.phase == Phase::FilesRemoved
        && cfg!(windows)
        && receipt.plan.core["owner"] != "unverified"
    {
        if let Err(error) = spawn_helper(target, receipt) {
            receipt.last_error = Some(error.message().into());
            save(target, receipt)?;
            result = Err(error);
        }
    }
    let complete = receipt.phase == Phase::Complete && result.is_ok();
    let ko = receipt.language == crate::user_setup::InterfaceLanguage::Ko;
    let mut output = match result {
        Ok(()) => ActionResult {
            schema_version: 1,
            action: "UninstallHiveUser",
            status: if complete { "success" } else { "pending" },
            exit_code: if complete { 0 } else { 2 },
            code: if complete {
                "hive.user-uninstall-complete"
            } else {
                "hive.user-uninstall-pending"
            },
            message: match (ko, complete) {
                (true, true) => "Hive 제거 확인 완료. 저장된 설정과 지식·프로젝트 자료 보존.",
                (true, false) => {
                    "아직 제거하지 못한 항목이 있습니다. 저장된 작업으로 이어서 진행할 수 있습니다."
                }
                (false, true) => {
                    "Hive removal verified. Saved settings, knowledge, and project files preserved."
                }
                (false, false) => {
                    "Removal is incomplete. Resume the saved operation to finish remaining steps."
                }
            }
            .into(),
            changed_paths: Vec::new(),
            evidence: Vec::new(),
            next_action: None,
            data: None,
        },
        Err(error) => failure("UninstallHiveUser", &error),
    };
    output.changed_paths.clone_from(&receipt.changed_paths);
    let helper = helper_path(&receipt.operation_id)?;
    let helper_exists =
        read_optional_regular(&target.root_cap, &helper, 128 * 1024 * 1024)?.is_some();
    let program = if helper_exists {
        format!(
            "{}{}",
            if cfg!(windows) { "& " } else { "" },
            quote(&target.user_root.join(&helper))
        )
    } else {
        "hive".into()
    };
    let command = format!(
        "{program} uninstall --user-root {} --resume {}",
        quote(&target.user_root),
        receipt.operation_id
    );
    if !complete {
        output.next_action = Some(command);
    }
    output.data = Some(
        json!({"operation_id":receipt.operation_id,"phase":receipt.phase,
        "completion_verified":complete,"preserved":PRESERVED,"receipt":path(&receipt.operation_id)?,
        "core":receipt.plan.core,"resume_command":output.next_action,"user_root":target.user_root,
        "helper":helper_exists.then_some(helper),"interface_language":receipt.language,
        "steps":{"monitor":if receipt.phase>=Phase::MonitorStopped {"stopped"}else{"unverified"},
            "host_activation":if receipt.phase>=Phase::HostsRemoved {"removed"}else{"unverified"},
            "files":if receipt.phase>=Phase::FilesRemoved {"removed"}else{"incomplete"},"core":if complete {"removed"}else{"unverified"}},
        "preserved_stop_receipts":[".hive/runtime/usage-monitor/config.json",".hive/runtime/usage-monitor/disable.json"],
        "retained_helper_purpose":helper_exists.then_some("verified result inspection and recovery; outside removal scope"),
        "diagnostic":receipt.last_error}),
    );
    Ok(output)
}

fn quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!(
        "'{}'",
        if cfg!(windows) {
            value.replace('\'', "''")
        } else {
            value.replace('\'', "'\"'\"'")
        }
    )
}

fn apply(target: &UserUninstallArguments, receipt: &mut Receipt) -> Result<(), InstallError> {
    if receipt.phase == Phase::Complete {
        return Ok(());
    }
    validate(target, receipt)?;
    if receipt.plan.core["owner"] != "unverified" {
        let executable =
            Path::new(receipt.plan.core["executable"].as_str().ok_or_else(|| {
                InstallError::Conflict("approved program path is missing".into())
            })?);
        if !executable.is_absolute() {
            return Err(InstallError::Conflict(
                "approved program path is not absolute".into(),
            ));
        }
        match std::fs::symlink_metadata(executable) {
            Ok(_) => {
                if crate::update_activation::uninstall_owner_preview(executable)
                    .map_err(InstallError::Conflict)?
                    != receipt.plan.core
                {
                    return Err(InstallError::Conflict(
                        "program changed after removal approval; no further removal was started"
                            .into(),
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(InstallError::Conflict(format!(
                    "cannot verify approved program: {error}"
                )))
            }
        }
    }
    let choice: crate::usage_monitor::SavedMonitorChoice =
        serde_json::from_value(receipt.plan.monitor.clone())
            .map_err(|_| InstallError::Conflict("invalid approved monitor selection".into()))?;
    let stopped = crate::usage_monitor::quiesce_for_uninstall(
        &target.user_root,
        &choice,
        &receipt.operation_id,
    )
    .map_err(|e| InstallError::Verification(e.message().into()))?;
    if receipt.phase == Phase::Approved {
        receipt.phase = Phase::MonitorStopped;
        save(target, receipt)?;
        interrupt("monitor-stopped");
    }
    if receipt.phase == Phase::MonitorStopped {
        receipt.changed_paths.extend(deactivation::apply_using(
            &target.user_root,
            &receipt.plan.native,
            &SystemCommandRunner,
        )?);
        receipt.phase = Phase::HostsRemoved;
        save(target, receipt)?;
        interrupt("hosts-removed");
    }
    if receipt.phase == Phase::HostsRemoved {
        remove_files(target, &receipt.plan, &mut receipt.changed_paths)?;
        interrupt("files-removed-before-checkpoint");
        receipt.phase = Phase::FilesRemoved;
        save(target, receipt)?;
    }
    stopped
        .verify_root(&target.user_root)
        .map_err(|e| InstallError::Conflict(e.message().into()))?;
    deactivation::verify_removed(&receipt.plan.native, &SystemCommandRunner)?;
    // Program removal follows only verified monitor shutdown and host deactivation.
    let removed = crate::update_activation::remove_owned_core(&receipt.plan.core)
        .map_err(InstallError::Verification)?;
    if removed {
        remove_monitor_payload(target, &receipt.plan, &mut receipt.changed_paths)?;
        receipt.phase = Phase::Complete;
        save(target, receipt)?;
    }
    Ok(())
}

fn interrupt(point: &str) {
    #[cfg(debug_assertions)]
    if std::env::var("HIVE_TEST_UNINSTALL_EXIT_AFTER").as_deref() == Ok(point) {
        std::process::exit(90);
    }
    #[cfg(not(debug_assertions))]
    let _ = point;
}

fn remove_monitor_payload(
    target: &UserUninstallArguments,
    plan: &Preview,
    changed: &mut Vec<String>,
) -> Result<(), InstallError> {
    // Keep the disabled selection, its stop proof and live lock files. They prevent an old
    // setup consent from becoming a new activation; the monitor executable and observations go.
    for (name, entry) in plan
        .entries
        .iter()
        .filter(|(name, _)| name.starts_with(".hive/runtime/usage-monitor/"))
    {
        let path = Path::new(name);
        if entry.kind == "directory" || entry.kind == "preserved" {
            continue;
        }
        let Some(bytes) = read_optional_regular(&target.root_cap, path, 128 * 1024 * 1024)? else {
            continue;
        };
        if entry
            .digest
            .as_ref()
            .is_some_and(|expected| *expected != sha256_digest(&bytes))
        {
            return Err(InstallError::Conflict(format!(
                "monitor payload changed after removal approval: {name}"
            )));
        }
        let permissions = file_permissions(&target.root_cap, path)?;
        remove_regular_if_exists(&target.root_cap, path, Some(&bytes), Some(permissions))?;
        changed.push(name.clone());
    }
    changed.sort();
    changed.dedup();
    Ok(())
}

fn helper_path(id: &str) -> Result<PathBuf, InstallError> {
    path(id)?;
    Ok(Path::new(".hive/maintenance/uninstall")
        .join(id)
        .join("hive.exe"))
}

fn spawn_helper(target: &UserUninstallArguments, receipt: &Receipt) -> Result<(), InstallError> {
    let source = env::current_exe().map_err(|e| InstallError::Internal(e.to_string()))?;
    let source = source
        .canonicalize()
        .map_err(|e| InstallError::Internal(e.to_string()))?;
    let original = PathBuf::from(
        receipt.plan.core["executable"]
            .as_str()
            .ok_or_else(|| InstallError::Conflict("approved executable is missing".into()))?,
    );
    // A helper never recursively launches another helper after a partial npm removal.
    if source != original {
        return Ok(());
    }
    let source_bytes = std::fs::read(&source).map_err(|e| InstallError::Internal(e.to_string()))?;
    if receipt.plan.core["executable_digest"] != sha256_digest(&source_bytes) {
        return Err(InstallError::Conflict(
            "program changed before removal handoff".into(),
        ));
    }
    let relative = helper_path(&receipt.operation_id)?;
    let previous = read_optional_regular(&target.root_cap, &relative, 128 * 1024 * 1024)?;
    if previous
        .as_deref()
        .is_some_and(|bytes| bytes != source_bytes)
    {
        return Err(InstallError::Conflict(
            "removal helper changed; preserved".into(),
        ));
    }
    if previous.is_none() {
        write_atomic(&target.root_cap, &relative, &source_bytes, true, None, None)?;
    }
    let mut command = std::process::Command::new(target.user_root.join(relative));
    command
        .args(["__complete-user-uninstall", "--user-root"])
        .arg(&target.user_root)
        .args(["--operation", &receipt.operation_id])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.spawn().map_err(|_| {
        InstallError::Verification(
            "removal helper could not start; saved removal remains pending".into(),
        )
    })?;
    Ok(())
}

pub(in crate::user_install) fn complete_helper(arguments: &[String]) -> ExitCode {
    let result = (|| {
        if arguments.len() != 4
            || arguments[0] != "--user-root"
            || arguments[2] != "--operation"
            || !cfg!(windows)
        {
            return Err(InstallError::Input("invalid removal helper request".into()));
        }
        let target = parse_uninstall(&arguments[..2])?;
        let relative = path(&arguments[3])?;
        let bytes =
            read_optional_regular(&target.root_cap, &relative, LIMIT)?.ok_or_else(|| {
                InstallError::Conflict("removal helper has no approved receipt".into())
            })?;
        let mut receipt: Receipt = serde_json::from_slice(&bytes)
            .map_err(|_| InstallError::Conflict("invalid removal helper receipt".into()))?;
        validate(&target, &receipt)?;
        if receipt.operation_id != arguments[3] {
            return Err(InstallError::Conflict(
                "removal helper operation differs".into(),
            ));
        }
        let own = env::current_exe()
            .and_then(|p| p.canonicalize())
            .map_err(|e| InstallError::Internal(e.to_string()))?;
        let expected = target.user_root.join(helper_path(&receipt.operation_id)?);
        if own != expected
            || sha256_digest(
                &std::fs::read(&own).map_err(|e| InstallError::Internal(e.to_string()))?,
            ) != receipt.plan.core["executable_digest"]
        {
            return Err(InstallError::Conflict(
                "removal helper does not match the approved program".into(),
            ));
        }
        let original = Path::new(
            receipt.plan.core["executable"]
                .as_str()
                .ok_or_else(|| InstallError::Conflict("missing program path".into()))?,
        );
        if original.exists() {
            if let Err(message) = crate::update_activation::wait_for_windows_direct_unlock(original)
            {
                if let Ok(_lease) = lock_user_setup(&target.root_cap) {
                    receipt.last_error = Some(message.clone());
                    save(&target, &receipt)?;
                }
                return Err(InstallError::Verification(message));
            }
        }
        resume(&target, &receipt.operation_id)
    })()
    .unwrap_or_else(|error| failure("UninstallHiveUser", &error));
    emit_action_result(&result)
}

fn remove_files(
    target: &UserUninstallArguments,
    plan: &Preview,
    changed: &mut Vec<String>,
) -> Result<(), InstallError> {
    let mut current = BTreeMap::new();
    for base in UNINSTALL_TREES {
        tree(&target.root_cap, Path::new(base), &mut current)?;
    }
    for name in current.keys() {
        if !plan.entries.contains_key(name)
            && !name.starts_with(".hive/runtime/usage-monitor")
            && name != ".hive/runtime"
        {
            return Err(InstallError::Conflict(format!(
                "new file or directory appeared after removal approval; preserved: {name}"
            )));
        }
    }
    for (name, entry) in &plan.entries {
        let path = Path::new(name);
        // Existing stop locks and their receipt stay valid while the controller is held.
        // They contain no user knowledge and cannot reactivate a retired monitor.
        if path.starts_with(".hive/runtime/usage-monitor") || entry.kind == "directory" {
            continue;
        }
        let Some(bytes) = read_optional_regular(&target.root_cap, path, 128 * 1024 * 1024)? else {
            continue;
        };
        let current = sha256_digest(&bytes);
        if entry.kind == "marker" && entry.after_digest.as_ref() == Some(&current) {
            continue;
        }
        if entry.digest.as_ref() != Some(&current)
            || entry.permissions != Some(file_permissions(&target.root_cap, path)?)
        {
            return Err(InstallError::Conflict(format!(
                "file changed after removal approval; preserved: {name}"
            )));
        }
        if entry.kind == "marker" {
            remove_hive_guidance_marker(&target.root_cap, path, changed)?;
        } else {
            remove_regular_if_exists(&target.root_cap, path, Some(&bytes), entry.permissions)?;
            changed.push(name.clone());
        }
    }
    let mut directories = plan
        .entries
        .iter()
        .filter(|(_, entry)| entry.kind == "directory")
        .map(|(path, _)| PathBuf::from(path))
        .collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for path in directories {
        if !path.starts_with(".hive/runtime/usage-monitor") {
            remove_owned_empty_dir(&target.root_cap, &path)?;
        }
    }
    prune_empty_owned_ancestors(
        &target.root_cap,
        plan.entries
            .iter()
            .filter(|(name, entry)| {
                entry.kind == "file" && !name.starts_with(".hive/runtime/usage-monitor")
            })
            .map(|(name, _)| Path::new(name)),
    )?;
    changed.sort();
    changed.dedup();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{preview, Entry};
    use super::*;
    use crate::user_install::open_user_root;
    use std::fs;

    #[test]
    fn resumed_removal_refuses_a_changed_program_before_stopping_or_deleting_files() {
        let root = tempfile::tempdir().unwrap();
        let user_root = root.path().canonicalize().unwrap();
        let target = UserUninstallArguments {
            root_cap: open_user_root(&user_root).unwrap(),
            user_root,
        };
        let derived = target.user_root.join(".hive/index/derived.db");
        fs::create_dir_all(derived.parent().unwrap()).unwrap();
        fs::write(&derived, b"keep until approval is current").unwrap();
        let executable = target.user_root.join("prefix/bin").join(if cfg!(windows) {
            "hive.exe"
        } else {
            "hive"
        });
        let receipt_path = target
            .user_root
            .join("prefix/share/aigent-hive/install-receipt.json");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::create_dir_all(receipt_path.parent().unwrap()).unwrap();
        fs::write(&executable, b"fixture old program bytes").unwrap();
        fs::write(
            &receipt_path,
            serde_json::to_vec(
                &json!({"schema_version":1,"owner":"direct","product":"aigent-hive",
            "version":env!("CARGO_PKG_VERSION"),"package_version":env!("CARGO_PKG_VERSION"),
            "artifact_sha256":sha256_digest(b"fixture old program bytes")}),
            )
            .unwrap(),
        )
        .unwrap();
        let plan = preview(&target, &executable).unwrap();
        assert_eq!(plan.core["owner"], "direct");
        let mut receipt = Receipt {
            schema_version: 1,
            operation_id: "a".repeat(64),
            product_version: env!("CARGO_PKG_VERSION").into(),
            preview_digest: digest(&plan).unwrap(),
            plan,
            phase: Phase::Approved,
            changed_paths: Vec::new(),
            language: crate::user_setup::InterfaceLanguage::En,
            last_error: None,
        };
        fs::write(executable, b"new program after approval").unwrap();
        assert!(apply(&target, &mut receipt).is_err());
        assert_eq!(
            fs::read(derived).unwrap(),
            b"keep until approval is current"
        );
        assert!(!target.user_root.join(".hive/runtime").exists());
        receipt.plan.core["executable"] = json!("relative-program.exe");
        receipt.preview_digest = digest(&receipt.plan).unwrap();
        assert!(apply(&target, &mut receipt).is_err());
        assert!(!target.user_root.join(".hive/runtime").exists());
    }

    #[test]
    fn approved_removal_preserves_knowledge_and_stays_pending_for_unknown_program_owner() {
        let root = tempfile::tempdir().unwrap();
        let user_root = root.path().canonicalize().unwrap();
        for dir in [".hive/index", ".hive/knowledge", ".hive/config"] {
            fs::create_dir_all(user_root.join(dir)).unwrap();
        }
        fs::write(user_root.join(".hive/index/derived.db"), b"derived").unwrap();
        fs::write(user_root.join(".hive/knowledge/fact.md"), b"user knowledge").unwrap();
        fs::write(
            user_root.join(".hive/config/projects.yml"),
            b"registered projects",
        )
        .unwrap();
        let target = UserUninstallArguments {
            root_cap: open_user_root(&user_root).unwrap(),
            user_root,
        };
        let plan = preview(&target, &env::current_exe().unwrap()).unwrap();
        let result = start(&target, plan, true).unwrap();
        assert_eq!(
            result.exit_code,
            2,
            "{}",
            serde_json::to_string(&result).unwrap()
        );
        assert_eq!(result.code, "hive.user-uninstall-pending");
        assert!(!target.user_root.join(".hive/index/derived.db").exists());
        assert_eq!(
            fs::read(target.user_root.join(".hive/knowledge/fact.md")).unwrap(),
            b"user knowledge"
        );
        assert_eq!(
            fs::read(target.user_root.join(".hive/config/projects.yml")).unwrap(),
            b"registered projects"
        );
        let data = result.data.unwrap();
        assert_eq!(data["steps"]["files"], "removed");
        assert_eq!(data["steps"]["core"], "unverified");
        assert_eq!(data["completion_verified"], false);
        let id = data["operation_id"].as_str().unwrap();
        let resumed = resume(&target, id).unwrap();
        assert_eq!(resumed.exit_code, 2);
        assert_eq!(resumed.data.unwrap()["operation_id"], id);
        let mut receipt = read(&target, id).unwrap();
        receipt.plan.entries.insert(
            "outside.txt".into(),
            Entry {
                kind: "file".into(),
                digest: Some(sha256_digest(b"outside")),
                after_digest: None,
                permissions: None,
            },
        );
        receipt.preview_digest = digest(&receipt.plan).unwrap();
        assert!(validate(&target, &receipt).is_err());
    }
}
