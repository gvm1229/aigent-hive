//! Confirmed setup can remove only the exact Hive registrations previewed earlier.
use super::{
    build_plan, expected_antigravity_plugin_state, expected_claude_marketplace_path,
    expected_codex_marketplace_root, expected_codex_plugin_source_path, file_permissions, find_all,
    host_uninstall_root, is_managed_ownership, open_canonical_user_root, owned_path_exists,
    probe_host_snapshot, probe_supported_host_version, read_installed_manifest,
    read_optional_regular, remove_regular_if_exists, validate_antigravity_prestate, write_atomic,
    HostStateSnapshot, InstallError, UserArguments, UserHost, UserMode, UserOwnershipManifest,
    COMMAND_OUTPUT_LIMIT, COMMAND_TIMEOUT, MAX_USER_FILE_BYTES, USER_MARKER_END, USER_MARKER_START,
};
use crate::usage::{CommandRunner, QualifiedExecutable, SystemCommandRunner};
use crate::user_setup::SelectedHost;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plan {
    pub(crate) host: SelectedHost,
    manifest_digest: String,
    before: HostStateSnapshot,
    guidance: String,
    guidance_before: String,
    guidance_after: String,
}

fn host(selected: SelectedHost) -> UserHost {
    match selected {
        SelectedHost::Codex => UserHost::Codex,
        SelectedHost::Claude => UserHost::Claude,
        SelectedHost::Antigravity => UserHost::Antigravity,
    }
}

fn digest(bytes: Option<&[u8]>) -> String {
    bytes.map_or_else(|| "absent".into(), sha256_digest)
}

pub(super) fn without_marker(bytes: &[u8]) -> Result<Vec<u8>, InstallError> {
    match (
        find_all(bytes, USER_MARKER_START).as_slice(),
        find_all(bytes, USER_MARKER_END).as_slice(),
    ) {
        ([], []) => Ok(bytes.to_vec()),
        ([start], [end]) if start < end => {
            let mut end = end + USER_MARKER_END.len();
            if bytes.get(end) == Some(&b'\n') {
                end += 1;
            }
            Ok([&bytes[..*start], &bytes[end..]].concat())
        }
        _ => Err(InstallError::Conflict(
            "Hive guidance marker is malformed".into(),
        )),
    }
}

fn executable(
    arguments: &UserArguments,
    runner: &impl CommandRunner,
) -> Result<QualifiedExecutable, InstallError> {
    let program = match arguments.host {
        UserHost::Codex => "codex",
        UserHost::Claude => "claude",
        UserHost::Antigravity => "agy",
    };
    let executable = runner.qualify(program).map_err(|_| {
        InstallError::Unsupported(format!(
            "{program} is unavailable; host selection preserved"
        ))
    })?;
    probe_supported_host_version(arguments.host, &executable, runner)?;
    Ok(executable)
}

fn verify_native_owner(
    arguments: &UserArguments,
    manifest: &UserOwnershipManifest,
    native: &HostStateSnapshot,
) -> Result<(), InstallError> {
    let valid = match native {
        HostStateSnapshot::Codex(state) => {
            let root = expected_codex_marketplace_root(arguments)?;
            let source = expected_codex_plugin_source_path(arguments)?;
            state
                .marketplace
                .as_ref()
                .is_none_or(|value| value.root == root)
                && state.plugin.as_ref().is_none_or(|value| {
                    state.marketplace.is_some()
                        && value.marketplace_source == root
                        && value.source_path == source
                        && value.version == manifest.product_version
                })
        }
        HostStateSnapshot::Claude(state) => {
            let path = expected_claude_marketplace_path(arguments)?;
            state
                .marketplace
                .as_ref()
                .is_none_or(|value| value.source == "directory" && value.path == path)
                && state.plugin.as_ref().is_none_or(|value| {
                    state.marketplace.is_some()
                        && value.scope == "user"
                        && value.version == manifest.product_version
                })
        }
        HostStateSnapshot::Antigravity(state) => state
            .plugin
            .as_ref()
            .is_none_or(|value| value == &expected_antigravity_plugin_state()),
    };
    if valid {
        Ok(())
    } else {
        Err(InstallError::Conflict(
            "Hive registration is not bound to the authenticated installed source; preserved"
                .into(),
        ))
    }
}

pub(crate) fn preview(root: &Path, removed: &[SelectedHost]) -> Result<Vec<Plan>, String> {
    preview_using(root, removed, &SystemCommandRunner).map_err(|e| e.message().into())
}

pub(super) fn preview_using(
    root: &Path,
    removed: &[SelectedHost],
    runner: &impl CommandRunner,
) -> Result<Vec<Plan>, InstallError> {
    if removed.is_empty() {
        return Ok(Vec::new());
    }
    let (root, cap) = open_canonical_user_root(root)?;
    let operational = crate::user_setup::resolved_operational_skills(&cap)
        .map_err(|e| InstallError::Conflict(e.message().into()))?;
    let mut plans = Vec::new();
    for selected in removed {
        let host = host(*selected);
        let path = PathBuf::from(format!(".hive/install/{}.json", host.as_str()));
        let Some(manifest) = read_installed_manifest(&cap, &path, host)? else {
            if owned_path_exists(&cap, host_uninstall_root(host))? {
                return Err(InstallError::Conflict(
                    "host source has no ownership manifest; preserved".into(),
                ));
            }
            continue;
        };
        let arguments = UserArguments {
            host,
            mode: UserMode::DryRun,
            user_root: root.clone(),
            root_cap: cap
                .try_clone()
                .map_err(|e| InstallError::Internal(e.to_string()))?,
            setup_override: operational.clone(),
        };
        let install = build_plan(&arguments)?;
        let exe = executable(&arguments, runner)?;
        let native = probe_host_snapshot(host, &exe, runner)?;
        verify_native_owner(&arguments, &manifest, &native)?;
        if let HostStateSnapshot::Antigravity(state) = &native {
            validate_antigravity_prestate(&arguments, &install, Some(state))?;
        }
        let bytes = read_optional_regular(
            &cap,
            Path::new(&manifest.guidance_path),
            MAX_USER_FILE_BYTES,
        )?;
        let after = without_marker(bytes.as_deref().unwrap_or_default())?;
        plans.push(Plan {
            host: *selected,
            manifest_digest: digest(
                read_optional_regular(&cap, &path, MAX_USER_FILE_BYTES)?.as_deref(),
            ),
            before: native,
            guidance: manifest.guidance_path,
            guidance_before: digest(bytes.as_deref()),
            guidance_after: digest((!after.is_empty()).then_some(after.as_slice())),
        });
    }
    Ok(plans)
}

fn transitions(before: &HostStateSnapshot) -> Vec<(&'static [&'static str], HostStateSnapshot)> {
    let mut steps = Vec::new();
    match before.clone() {
        HostStateSnapshot::Codex(mut state) => {
            if state.plugin.take().is_some() {
                steps.push((
                    &["plugin", "remove", "aigent-hive@aigent-hive", "--json"][..],
                    HostStateSnapshot::Codex(state.clone()),
                ));
            }
            if state.marketplace.take().is_some() {
                steps.push((
                    &["plugin", "marketplace", "remove", "aigent-hive", "--json"][..],
                    HostStateSnapshot::Codex(state),
                ));
            }
        }
        HostStateSnapshot::Claude(mut state) => {
            if state.plugin.take().is_some() {
                steps.push((
                    &[
                        "plugin",
                        "uninstall",
                        "aigent-hive@aigent-hive",
                        "--scope",
                        "user",
                    ][..],
                    HostStateSnapshot::Claude(state.clone()),
                ));
            }
            if state.marketplace.take().is_some() {
                steps.push((
                    &[
                        "plugin",
                        "marketplace",
                        "remove",
                        "aigent-hive",
                        "--scope",
                        "user",
                    ][..],
                    HostStateSnapshot::Claude(state),
                ));
            }
        }
        HostStateSnapshot::Antigravity(mut state) => {
            if state.plugin.take().is_some() {
                steps.push((
                    &["plugin", "uninstall", "aigent-hive"][..],
                    HostStateSnapshot::Antigravity(state),
                ));
            }
        }
    }
    steps
}

pub(crate) fn apply(root: &Path, plans: &[Plan]) -> Result<Vec<String>, String> {
    apply_using(root, plans, &SystemCommandRunner).map_err(|e| e.message().into())
}

pub(super) fn verify_removed(plans: &[Plan], runner: &impl CommandRunner) -> Result<(), InstallError> {
    for plan in plans {
        let host = host(plan.host);
        let program = match host {UserHost::Codex=>"codex",UserHost::Claude=>"claude",UserHost::Antigravity=>"agy"};
        let exe = runner.qualify(program).map_err(|_|InstallError::Unsupported("cannot verify removed host activation".into()))?;
        probe_supported_host_version(host,&exe,runner)?;
        let absent=match probe_host_snapshot(host,&exe,runner)? {
            HostStateSnapshot::Codex(state)=>state.plugin.is_none() && state.marketplace.is_none(),
            HostStateSnapshot::Claude(state)=>state.plugin.is_none() && state.marketplace.is_none(),
            HostStateSnapshot::Antigravity(state)=>state.plugin.is_none(),
        };
        if !absent {return Err(InstallError::Conflict("host activation changed during removal; program preserved".into()));}
    }
    Ok(())
}

pub(super) fn apply_using(
    root: &Path,
    plans: &[Plan],
    runner: &impl CommandRunner,
) -> Result<Vec<String>, InstallError> {
    if plans.is_empty() {
        return Ok(Vec::new());
    }
    let (root, cap) = open_canonical_user_root(root)?;
    let mut changed = Vec::new();
    for plan in plans {
        let host = host(plan.host);
        let manifest_path = PathBuf::from(format!(".hive/install/{}.json", host.as_str()));
        let bytes = read_optional_regular(&cap, &manifest_path, MAX_USER_FILE_BYTES)?;
        if digest(bytes.as_deref()) != plan.manifest_digest {
            return Err(InstallError::Conflict(
                "host ownership changed after preview".into(),
            ));
        }
        let manifest = read_installed_manifest(&cap, &manifest_path, host)?
            .ok_or_else(|| InstallError::Conflict("host ownership disappeared".into()))?;
        for entry in manifest
            .entries
            .iter()
            .filter(|entry| is_managed_ownership(&entry.ownership))
        {
            let actual = read_optional_regular(&cap, Path::new(&entry.path), MAX_USER_FILE_BYTES)?;
            if actual.as_deref().map(sha256_digest).as_ref() != Some(&entry.digest) {
                return Err(InstallError::Conflict(
                    "host source changed after preview; preserved".into(),
                ));
            }
        }
        if plan.guidance != manifest.guidance_path {
            return Err(InstallError::Conflict("host guidance scope changed".into()));
        }
        let guidance = read_optional_regular(&cap, Path::new(&plan.guidance), MAX_USER_FILE_BYTES)?;
        let fingerprint = digest(guidance.as_deref());
        if fingerprint != plan.guidance_before && fingerprint != plan.guidance_after {
            return Err(InstallError::Conflict(
                "host guidance changed after preview; preserved".into(),
            ));
        }
        let arguments = UserArguments {
            host,
            mode: UserMode::Apply,
            user_root: root.clone(),
            root_cap: cap
                .try_clone()
                .map_err(|e| InstallError::Internal(e.to_string()))?,
            setup_override: None,
        };
        let exe = executable(&arguments, runner)?;
        let mut observed = probe_host_snapshot(host, &exe, runner)?;
        let steps = transitions(&plan.before);
        let start = if observed == plan.before {
            0
        } else {
            steps
                .iter()
                .position(|(_, state)| state == &observed)
                .map(|index| index + 1)
                .ok_or_else(|| {
                    InstallError::Conflict(
                        "host registration changed after preview; preserved".into(),
                    )
                })?
        };
        for (command, after) in steps.iter().skip(start) {
            if probe_host_snapshot(host, &exe, runner)? != observed {
                return Err(InstallError::Conflict(
                    "host registration changed before deactivation".into(),
                ));
            }
            let output = runner
                .run(&exe, command, COMMAND_TIMEOUT, COMMAND_OUTPUT_LIMIT)
                .map_err(|e| {
                    InstallError::Verification(format!("host deactivation incomplete: {e}"))
                })?;
            observed = probe_host_snapshot(host, &exe, runner)?;
            if !output.success || observed != *after {
                return Err(InstallError::Verification(
                    "host deactivation incomplete; resume the saved setup".into(),
                ));
            }
        }
        if fingerprint == plan.guidance_before && fingerprint != plan.guidance_after {
            let before = guidance.as_deref().unwrap_or_default();
            let after = without_marker(before)?;
            let path = Path::new(&plan.guidance);
            let permissions = file_permissions(&cap, path)?;
            if after.is_empty() {
                remove_regular_if_exists(&cap, path, Some(before), Some(permissions))?;
            } else {
                write_atomic(&cap, path, &after, false, Some(before), Some(permissions))?;
            }
            changed.push(plan.guidance.clone());
        }
    }
    Ok(changed)
}
