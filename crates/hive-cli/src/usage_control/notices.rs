//! Optional notices never poll and never block work when their derived state is unavailable.
use super::{InstalledUsageConfig, PinnedTarget};
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod quota;
pub(super) use quota::observe as observe_quota;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FailureEpisode {
    schema_version: u8,
    identity_digest: String,
    policy_version: String,
    active: bool,
}

fn failure_path(config: &InstalledUsageConfig, account: Option<&str>) -> (String, PathBuf) {
    let identity = sha256_digest(
        serde_json::json!(["observation-failure", config.primary_host, account])
            .to_string()
            .as_bytes(),
    );
    let path = Path::new(".hive/runtime/usage-notices").join(format!("{}.json", &identity[7..]));
    (identity, path)
}

pub(super) fn claim_failure(
    user_root: Option<&Path>,
    fallback: &PinnedTarget,
    config: &InstalledUsageConfig,
    account: Option<&str>,
) -> bool {
    if crate::display_options::quiet()
        || !config
            .policy
            .as_ref()
            .is_some_and(|policy| policy.warnings_enabled)
    {
        return false;
    }
    change_failure(user_root, fallback, config, account, true).unwrap_or(false)
}

pub(super) fn recover_failure(
    user_root: Option<&Path>,
    fallback: &PinnedTarget,
    config: &InstalledUsageConfig,
    requested: Option<&str>,
    observed: Option<&str>,
) {
    if config.policy.is_none() {
        return;
    }
    for account in [requested, observed, None] {
        let _ = change_failure(user_root, fallback, config, account, false);
    }
}

fn change_failure(
    user_root: Option<&Path>,
    fallback: &PinnedTarget,
    config: &InstalledUsageConfig,
    account: Option<&str>,
    active: bool,
) -> Result<bool, super::AdapterError> {
    let owned = user_root.map(PinnedTarget::open_usage).transpose()?;
    let root = owned.as_ref().unwrap_or(fallback);
    let (identity, path) = failure_path(config, account);
    let snapshot = root.snapshot_bounded(&path, 4096)?;
    let old: Option<FailureEpisode> = snapshot
        .bytes()
        .map(serde_json::from_slice)
        .transpose()
        .map_err(|_| super::AdapterError::Safety("invalid warning episode".to_owned()))?;
    let version = concat!(env!("CARGO_PKG_VERSION"), ":observation-failure-v1");
    if let Some(old) = &old {
        if old.schema_version != 1 || old.identity_digest != identity {
            return Err(super::AdapterError::Safety(
                "warning episode identity differs".to_owned(),
            ));
        }
        if old.active == active && (!active || old.policy_version == version) {
            return Ok(false);
        }
    } else if !active {
        return Ok(false);
    }
    let next = FailureEpisode {
        schema_version: 1,
        identity_digest: identity,
        policy_version: version.to_owned(),
        active,
    };
    let bytes = serde_json::to_vec(&next)
        .map_err(|error| super::AdapterError::Internal(error.to_string()))?;
    root.verify_current()?;
    crate::user_install::replace_user_setup_file(
        root.target_dir(),
        &path,
        snapshot.bytes(),
        Some(&bytes),
    )
    .map_err(super::AdapterError::Conflict)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simultaneous_failure_claims_emit_only_one_notice() {
        let user = tempfile::tempdir().unwrap();
        let mut config = super::super::parse_installed_config(
            super::super::tests::installed_config("codex", true, false),
        )
        .unwrap();
        config.policy = Some(hive_core::preferences::UsageProtection::default());
        let barrier = std::sync::Barrier::new(2);
        let results = std::thread::scope(|scope| {
            let claim = || {
                let root = PinnedTarget::open_usage(user.path()).unwrap();
                barrier.wait();
                claim_failure(Some(user.path()), &root, &config, None)
            };
            let first = scope.spawn(claim);
            let second = scope.spawn(claim);
            [first.join().unwrap(), second.join().unwrap()]
        });
        assert_eq!(results.into_iter().filter(|shown| *shown).count(), 1);
    }
}
