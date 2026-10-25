//! An explicit observation never enforces policy, grants dispatch or writes Hive state.
use super::{native_then_consented_fallback, AdapterError, PinnedTarget, USER_CONFIG_PATH};
use crate::{
    usage::{self, NormalizedSnapshot, SensorError, UsageHost},
    user_setup::InterfaceLanguage,
    ActionResult, Evidence,
};
use hive_core::usage_guard::{evaluate_observation_only, UsageDecision, UsagePolicy};
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct Options {
    root: PathBuf,
    host: UsageHost,
    account: Option<String>,
}

fn parse(arguments: &[String]) -> Result<Options, AdapterError> {
    if !arguments.len().is_multiple_of(2) {
        return Err(AdapterError::Input("usage show option has no value".into()));
    }
    let mut values = BTreeMap::new();
    for pair in arguments.chunks_exact(2) {
        if !matches!(
            pair[0].as_str(),
            "--host" | "--user-root" | "--account-digest" | "--output"
        ) || pair[1].starts_with("--")
            || values.insert(pair[0].as_str(), pair[1].as_str()).is_some()
        {
            return Err(AdapterError::Input(
                "unknown or duplicate usage show option".into(),
            ));
        }
    }
    if values.get("--output") != Some(&"json") {
        return Err(AdapterError::Input(
            "usage show requires --output json".into(),
        ));
    }
    let host = match values.get("--host").copied() {
        Some("codex") => UsageHost::Codex,
        Some("claude") => UsageHost::Claude,
        Some("antigravity") => UsageHost::Antigravity,
        _ => {
            return Err(AdapterError::Input(
                "usage show requires --host codex|claude|antigravity".into(),
            ))
        }
    };
    let root = values
        .get("--user-root")
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
        .ok_or_else(|| AdapterError::Input("usage show requires an absolute --user-root".into()))?;
    let account = values
        .get("--account-digest")
        .map(|value| (*value).to_owned());
    if account
        .as_ref()
        .is_some_and(|value| !crate::is_sha256_digest(value))
    {
        return Err(AdapterError::Input(
            "usage show account digest must be SHA256".into(),
        ));
    }
    Ok(Options {
        root,
        host,
        account,
    })
}

pub(super) fn run(arguments: &[String]) -> Result<ActionResult, AdapterError> {
    observe(&parse(arguments)?, SystemTime::now(), read)
}

fn read(
    host: UsageHost,
    fallback: bool,
    account: Option<&str>,
    now: SystemTime,
) -> Result<NormalizedSnapshot, SensorError> {
    let native = match host {
        UsageHost::Codex => {
            usage::NativeUsageRunner::read_codex_native(&usage::SystemCommandRunner, account, now)
        }
        UsageHost::Claude | UsageHost::Antigravity => Err(SensorError::Unavailable),
    };
    native_then_consented_fallback(host, native, fallback, || match account {
        Some(account) => usage::check_codexbar_provider_with_runner(
            &usage::SystemCommandRunner,
            host,
            account,
            now,
        ),
        None => usage::check_codexbar_provider_unique_with_runner(
            &usage::SystemCommandRunner,
            host,
            now,
        ),
    })
}

fn observe(
    options: &Options,
    now: SystemTime,
    sample: impl FnOnce(
        UsageHost,
        bool,
        Option<&str>,
        SystemTime,
    ) -> Result<NormalizedSnapshot, SensorError>,
) -> Result<ActionResult, AdapterError> {
    let root = PinnedTarget::open_usage(&options.root)?;
    let before = crate::user_install::read_user_setup_file(
        root.target_dir(),
        Path::new(USER_CONFIG_PATH),
        crate::user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(AdapterError::Safety)?
    .ok_or_else(|| AdapterError::Input("global Hive setup is required for manual usage".into()))?;
    let config = crate::user_setup::parse_and_validate_installed_config(&before)
        .map_err(|error| AdapterError::Safety(error.message().into()))?;
    if !config
        .selected_hosts
        .iter()
        .any(|host| host.as_str() == options.host.as_str())
    {
        return Err(AdapterError::Input(
            "manual usage host is not selected in this user setup".into(),
        ));
    }
    let at = now
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|t| i64::try_from(t.as_secs()).ok())
        .ok_or_else(|| AdapterError::Internal("cannot validate observation time".into()))?;
    // Manual observation uses the saved fallback consent, independent of enforcement switches.
    let observation = sample(
        options.host,
        config.usage_guard.codexbar_fallback_enabled,
        options.account.as_deref(),
        now,
    )
    .and_then(|snapshot| {
        if options
            .account
            .as_ref()
            .is_some_and(|account| account != &snapshot.account_digest)
        {
            return Err(SensorError::AccountNotFound);
        }
        let policy = UsagePolicy::new(
            &snapshot.sensor_id,
            &snapshot.sensor_version,
            options.host.as_str(),
            &snapshot.account_digest,
        );
        if !matches!(
            evaluate_observation_only(&policy, &snapshot.core_snapshots(), at),
            UsageDecision::Allow(_)
        ) {
            return Err(SensorError::Malformed);
        }
        Ok(snapshot)
    });
    root.verify_current()?;
    if crate::user_install::read_user_setup_file(
        root.target_dir(),
        Path::new(USER_CONFIG_PATH),
        crate::user_setup::MAX_USER_SETUP_BYTES,
    )
    .map_err(AdapterError::Safety)?
    .as_deref()
        != Some(before.as_slice())
    {
        return Err(AdapterError::Conflict(
            "manual usage settings changed during observation".into(),
        ));
    }
    Ok(observation_result(
        config.interface_language,
        options.host,
        observation,
    ))
}

fn observation_result(
    language: InterfaceLanguage,
    host: UsageHost,
    observation: Result<NormalizedSnapshot, SensorError>,
) -> ActionResult {
    let korean = language == InterfaceLanguage::Ko;
    let mut result = ActionResult {
        schema_version: 1,
        action: "ShowUsageStatus",
        status: "success",
        exit_code: 0,
        code: "hive.usage-observed",
        message: if korean {
            "사용량을 확인했습니다. 보호 설정과 작업 상태는 변경하지 않았습니다."
        } else {
            "Usage observed. Protection settings and work state are unchanged."
        }
        .into(),
        changed_paths: vec![],
        evidence: vec![],
        next_action: None,
        data: Some(json!({"scope":"manual-observation","host":host.as_str(),
            "authorizes_dispatch":false,"writes_hive_state":false,"model_calls":0})),
    };
    match observation {
        Ok(snapshot) => {
            result.evidence.push(Evidence {
                kind: "report",
                locator: "usage-snapshots:manual".into(),
                digest: snapshot.evidence_digest(),
            });
            result.data.as_mut().expect("manual data")["snapshot"] = json!(snapshot);
        }
        Err(error) => {
            result.status = "error";
            result.exit_code = 2;
            result.code = "hive.usage-observation-unavailable";
            result.message = if korean {
                "사용량을 확인하지 못했습니다. 보호 설정과 작업 상태는 그대로입니다."
            } else {
                "Usage is unavailable. Protection settings and work state are unchanged."
            }
            .into();
            result.data.as_mut().expect("manual data")["reason"] = json!(error.to_string());
        }
    }
    result
}

pub(crate) fn command(root: &Path, host: &str) -> String {
    let path = root.to_string_lossy();
    let quoted = if cfg!(windows) {
        path.replace('\'', "''")
    } else {
        path.replace('\'', "'\"'\"'")
    };
    format!("hive usage show --host {host} --user-root '{quoted}' --output json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::time::Duration;

    fn fixture() -> (tempfile::TempDir, Options, Vec<u8>) {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(USER_CONFIG_PATH);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = br#"{"schema_version":2,"interface_language":"ko","wiki":{"language":"ko"},"profile":{"contexts":["non-developer"]},"persona":{"id":"balanced"},"selected_hosts":["codex"],"skills":{"mode":"all"},"usage_guard":{"enforcement_enabled":false,"warnings_enabled":true,"stop_remaining_percent":20,"codexbar_fallback_enabled":true,"preflight":{"threshold_enabled":false,"reset_enabled":false,"zero_quota_enabled":false,"observation_failure":"warn"},"midway":{"enabled":false,"threshold_enabled":false,"reset_enabled":false,"zero_quota_enabled":false,"observation_failure":"warn"}}}"#.to_vec();
        std::fs::write(path, &bytes).unwrap();
        let options = Options {
            root: root.path().into(),
            host: UsageHost::Codex,
            account: None,
        };
        (root, options, bytes)
    }
    fn snapshot() -> NormalizedSnapshot {
        NormalizedSnapshot {
            sensor_id: "codex-native".into(),
            sensor_version: "1.0.0".into(),
            provider: "codex".into(),
            account_digest: hive_core::sha256_digest(b"fixture account"),
            measured_at: 1000,
            expires_at: 1120,
            source_confidence: "high".into(),
            windows: vec![usage::NormalizedWindow {
                name: "session",
                quota_pool: None,
                window_minutes: Some(300),
                remaining_percent: 0.0,
                resets_at: 2000,
            }],
        }
    }
    #[test]
    fn manual_zero_observation_succeeds_with_protection_off_and_keeps_all_hive_state() {
        let (root, options, before) = fixture();
        let calls = Cell::new(0);
        let result = observe(
            &options,
            UNIX_EPOCH + Duration::from_secs(1000),
            |host, fallback, account, _| {
                calls.set(calls.get() + 1);
                assert_eq!(host, UsageHost::Codex);
                assert!(fallback);
                assert!(account.is_none());
                Ok(snapshot())
            },
        )
        .unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(result.code, "hive.usage-observed");
        assert_eq!(result.exit_code, 0);
        let data = result.data.unwrap();
        assert_eq!(data["snapshot"]["windows"][0]["remaining_percent"], 0.0);
        assert_eq!(data["authorizes_dispatch"], false);
        assert_eq!(data["writes_hive_state"], false);
        assert!(result.changed_paths.is_empty());
        assert_eq!(
            std::fs::read(root.path().join(USER_CONFIG_PATH)).unwrap(),
            before
        );
        assert!(!root.path().join(".hive/runtime").exists());
    }
    #[test]
    fn manual_unavailable_stale_wrong_host_and_wrong_account_never_claim_an_observation() {
        let (root, mut options, before) = fixture();
        let at = UNIX_EPOCH + Duration::from_secs(1000);
        let mut stale = snapshot();
        stale.expires_at = 900;
        let mut other = snapshot();
        other.provider = "claude".into();
        for outcome in [Err(SensorError::Unavailable), Ok(stale), Ok(other)] {
            let result = observe(&options, at, |_, _, _, _| outcome).unwrap();
            assert_eq!(result.exit_code, 2);
            assert_eq!(result.code, "hive.usage-observation-unavailable");
            assert!(result.evidence.is_empty());
            assert!(result.data.unwrap().get("snapshot").is_none());
        }
        options.account = Some(hive_core::sha256_digest(b"different"));
        assert_eq!(
            observe(&options, at, |_, _, _, _| Ok(snapshot()))
                .unwrap()
                .exit_code,
            2
        );
        assert_eq!(
            std::fs::read(root.path().join(USER_CONFIG_PATH)).unwrap(),
            before
        );
        assert!(!root.path().join(".hive/runtime").exists());
        options.host = UsageHost::Claude;
        assert!(observe(&options, at, |_, _, _, _| panic!("unselected host sampled")).is_err());
    }
    #[test]
    fn manual_parser_rejects_ambiguous_scope_and_invalid_account_before_observation() {
        let root = tempfile::tempdir().unwrap();
        let base = vec![
            "--host".into(),
            "codex".into(),
            "--user-root".into(),
            root.path().display().to_string(),
            "--output".into(),
            "json".into(),
        ];
        assert!(parse(&base).is_ok());
        for tail in [
            vec!["--host", "claude"],
            vec!["--account-digest", "raw-identity"],
            vec!["--unknown", "yes"],
            vec!["--account-digest"],
        ] {
            let args = base
                .iter()
                .cloned()
                .chain(tail.into_iter().map(str::to_owned))
                .collect::<Vec<_>>();
            assert!(parse(&args).is_err());
        }
        assert!(!root.path().join(".hive").exists());
    }
}
