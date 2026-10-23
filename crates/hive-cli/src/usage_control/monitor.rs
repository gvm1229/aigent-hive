//! Policy sharing and process-bound overrides for the opt-in live monitor.
use super::{
    bind_runtime_session, effective_enabled, effective_policy_digest, effective_zero_quota_enabled,
    evaluate_guard_decision, load_control, read_effective_config, AdapterError, EnforceArguments,
    InstalledUsageConfig, LoadedControl, OverrideState, ParsedBinding, Path, PathBuf, PinnedTarget,
    SessionBinding,
};
use hive_core::{
    sha256_digest,
    usage_guard::{UsageBlock, UsageDecision, UsagePolicy, UsageSnapshot},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub(crate) struct MonitorPolicy {
    pub session_digest: String,
    pub owner_digest: String,
    pub target_digest: String,
    pub policy_digest: String,
    pub control_digest: String,
    pub threshold: u8,
    pub enabled: bool,
    pub zero_enabled: bool,
    pub turn_digest: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    schema_version: u8,
    session_digest: String,
    target_digest: String,
    process_id: u32,
    process_identity: String,
    control_digest: String,
}

fn binding_path(target: &str, session: &str) -> PathBuf {
    Path::new(".hive/runtime/usage-monitor/bindings").join(format!(
        "{}-{}.json",
        target.trim_start_matches("sha256:"),
        session.trim_start_matches("sha256:")
    ))
}

fn from_config(
    config: &InstalledUsageConfig,
    binding: &SessionBinding,
    loaded: &LoadedControl,
    target: &PinnedTarget,
) -> Result<MonitorPolicy, AdapterError> {
    Ok(MonitorPolicy {
        session_digest: binding.session_digest.clone(),
        owner_digest: binding.session_digest.clone(),
        target_digest: sha256_digest(target.requested_path().to_string_lossy().as_bytes()),
        policy_digest: effective_policy_digest(config)?,
        control_digest: loaded
            .snapshot
            .bytes()
            .map_or_else(|| "absent".into(), sha256_digest),
        threshold: config.threshold,
        enabled: effective_enabled(loaded, config.guard_enabled),
        zero_enabled: effective_zero_quota_enabled(loaded),
        turn_digest: None,
    })
}

pub(super) fn current_policy(
    config: &InstalledUsageConfig,
    binding: &SessionBinding,
    loaded: &LoadedControl,
    target: &PinnedTarget,
) -> Result<MonitorPolicy, AdapterError> {
    from_config(config, binding, loaded, target)
}

/// The cwd comes from authenticated metadata. No consumer marker or configuration is created.
pub(crate) fn monitor_policy(
    user_root: &Path,
    cwd: &Path,
    session: &str,
) -> Result<Option<MonitorPolicy>, AdapterError> {
    let scope = cwd.ancestors().take(64).find(|path| {
        path.join("hive-source.json").is_file() || path.join(super::CONFIG_PATH).is_file()
    });
    let Some(scope) = scope else {
        return Ok(None);
    };
    let target = PinnedTarget::open_usage(scope)?;
    let runtime = PinnedTarget::open_usage(user_root)?;
    let config = read_effective_config(&target, Some(user_root), Some("codex"))?;
    if config
        .policy
        .as_ref()
        .is_some_and(|policy| !policy.effective_midway().enabled)
    {
        return Ok(None);
    }
    let mut binding = bind_runtime_session(
        &ParsedBinding {
            session_id: session.to_owned(),
            process_id: 0,
        },
        "codex",
        &target,
        true,
    );
    let loaded = load_control(&runtime, &binding)?;
    if let Some(control) = loaded.control.as_ref() {
        let target_digest = sha256_digest(target.requested_path().to_string_lossy().as_bytes());
        let path = binding_path(&target_digest, &binding.session_digest);
        let saved = runtime
            .read_optional(&path, 4096)?
            .and_then(|bytes| serde_json::from_slice::<Binding>(&bytes).ok());
        if let Some(saved) = saved.filter(|saved| {
            saved.schema_version == 1
                && saved.session_digest == binding.session_digest
                && saved.target_digest == target_digest
                && saved.process_id == control.process_id
                && saved.control_digest
                    == loaded
                        .snapshot
                        .bytes()
                        .map_or_else(|| "absent".into(), sha256_digest)
        }) {
            if crate::usage_monitor::session_process_identity(saved.process_id).as_deref()
                == Some(saved.process_identity.as_str())
            {
                binding.process_id = saved.process_id;
            }
        }
        if binding.process_id == 0 && (!control.guard_enabled || control.zero_quota_guard_disabled)
        {
            return Err(AdapterError::Safety(
                "monitor-session-binding-unverified".into(),
            ));
        }
    }
    let loaded = load_control(&runtime, &binding)?;
    from_config(&config, &binding, &loaded, &target).map(Some)
}

pub(crate) fn evaluate_monitor_policy(
    policy: &MonitorPolicy,
    snapshots: &[UsageSnapshot],
    at: u64,
) -> Result<Option<UsageBlock>, AdapterError> {
    let Some(first) = snapshots.first() else {
        return Err(AdapterError::Safety("monitor-observation-empty".into()));
    };
    let core = UsagePolicy::new(
        &first.sensor_id,
        &first.sensor_version,
        &first.host_scope,
        &first.account_scope_digest,
    )
    .with_stop_remaining_percent(policy.threshold)
    .map_err(|_| AdapterError::Safety("monitor-threshold-invalid".into()))?;
    match evaluate_guard_decision(
        &core,
        snapshots,
        i64::try_from(at).unwrap_or(i64::MAX),
        policy.enabled,
        policy.zero_enabled,
    ) {
        UsageDecision::Block(block) => Ok(Some(block)),
        UsageDecision::Allow(_) => Ok(None),
        UsageDecision::Unknown(_) => Err(AdapterError::Safety(
            "monitor-policy-observation-invalid".into(),
        )),
    }
}

pub(super) fn register(
    root: &PinnedTarget,
    target: &PinnedTarget,
    binding: &SessionBinding,
    control_bytes: &[u8],
    config: &InstalledUsageConfig,
) -> Result<Option<String>, AdapterError> {
    if config
        .policy
        .as_ref()
        .is_some_and(|policy| !policy.effective_midway().enabled)
    {
        return Ok(None);
    }
    if !crate::usage_monitor::monitor_enabled(root) || binding.host_scope != "codex" {
        return Ok(None);
    }
    let Some(process_identity) = crate::usage_monitor::session_process_identity(binding.process_id)
    else {
        return Ok(None);
    };
    let target_digest = sha256_digest(target.requested_path().to_string_lossy().as_bytes());
    let path = binding_path(&target_digest, &binding.session_digest);
    let bytes = serde_json::to_vec(&Binding {
        schema_version: 1,
        session_digest: binding.session_digest.clone(),
        target_digest,
        process_id: binding.process_id,
        process_identity,
        control_digest: sha256_digest(control_bytes),
    })
    .map_err(|_| AdapterError::Internal("monitor binding serialization failed".into()))?;
    let before = root.snapshot(&path)?;
    let changed = root.publish_runtime(&path, &before, &bytes)?;
    Ok(changed.then(|| super::portable_relative_path(&path)))
}

pub(super) fn register_for_arguments(
    arguments: &EnforceArguments,
) -> Result<Option<String>, AdapterError> {
    let Some(root) = arguments.user_root.as_deref() else {
        return Ok(None);
    };
    let target = PinnedTarget::open_usage(&arguments.target)?;
    let config = read_effective_config(&target, Some(root), arguments.host.as_deref())?;
    if config
        .policy
        .as_ref()
        .is_some_and(|policy| !policy.effective_midway().enabled)
    {
        return Ok(None);
    }
    let runtime = PinnedTarget::open_usage(root)?;
    if !crate::usage_monitor::monitor_enabled(&runtime) {
        return Ok(None);
    }
    let binding = bind_runtime_session(&arguments.binding, &config.primary_host, &target, true);
    let loaded = load_control(&runtime, &binding)?;
    if loaded.state != OverrideState::Current {
        return Ok(None);
    }
    match loaded.snapshot.bytes() {
        Some(bytes) => register(&runtime, &target, &binding, bytes, &config),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage_control::tests::{installed_config, write_global_config};

    #[test]
    fn live_policy_reuses_the_global_project_maximum_and_leaves_unconfigured_folders_alone() {
        let root = tempfile::tempdir().unwrap();
        write_global_config(root.path(), 10);
        let project = tempfile::tempdir().unwrap();
        let config = project.path().join(".hive/config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("harness.toml"),
            installed_config("codex", true, false),
        )
        .unwrap();
        let nested = project.path().join("src/nested");
        std::fs::create_dir_all(&nested).unwrap();
        let policy = monitor_policy(root.path(), &nested, "test-conversation")
            .unwrap()
            .unwrap();
        assert_eq!(policy.threshold, 20);
        write_global_config(root.path(), 30);
        assert_eq!(
            monitor_policy(root.path(), &nested, "test-conversation")
                .unwrap()
                .unwrap()
                .threshold,
            30
        );
        let unrelated = tempfile::tempdir().unwrap();
        assert!(
            monitor_policy(root.path(), unrelated.path(), "test-conversation")
                .unwrap()
                .is_none()
        );
        assert!(!unrelated.path().join(".hive").exists());
    }

    #[test]
    fn source_policy_uses_global_without_creating_source_consumer_state() {
        let root = tempfile::tempdir().unwrap();
        write_global_config(root.path(), 17);
        let source = tempfile::tempdir().unwrap();
        std::fs::write(source.path().join("hive-source.json"), b"{}").unwrap();
        let policy = monitor_policy(root.path(), source.path(), "source-conversation")
            .unwrap()
            .unwrap();
        assert_eq!(policy.threshold, 17);
        assert!(policy.enabled);
        assert!(!source.path().join(".hive").exists());
    }
}
