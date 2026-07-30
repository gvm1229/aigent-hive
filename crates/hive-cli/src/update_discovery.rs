use super::{emit_action_result, ActionResult, Evidence};
use hive_core::sha256_digest;
use hive_update::SemVersion;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REGISTRY_METADATA_URL: &str = "https://registry.npmjs.org/aigent-hive";
const UPDATE_CHECK_STATE_RELATIVE: &str = ".hive/runtime/update-check.json";
const MAX_STATE_BYTES: u64 = 64 * 1024;
const MAX_METADATA_BYTES: u64 = 1024 * 1024;
const SUCCESS_THROTTLE_SECONDS: i64 = 24 * 60 * 60;
const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);

const UPDATE_CHECK_USAGE: &str = "\
Check the fixed npm registry metadata endpoint for a newer Aigent Hive version.

USAGE:
    hive update --check --user-root <absolute-dir> --output json

The check never installs or activates an update.
";

#[derive(Debug)]
struct Arguments {
    user_root: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateCheckState {
    schema_version: u32,
    last_success_unix: i64,
    latest_version: String,
}

trait MetadataClient {
    fn fetch(&self) -> Result<Vec<u8>, String>;
}

struct RegistryClient;

impl MetadataClient for RegistryClient {
    fn fetch(&self) -> Result<Vec<u8>, String> {
        let config = ureq::Agent::config_builder()
            .https_only(true)
            .timeout_global(Some(NETWORK_TIMEOUT))
            .user_agent(concat!("aigent-hive/", env!("CARGO_PKG_VERSION")))
            .build();
        let agent: ureq::Agent = config.into();
        let mut response = agent
            .get(REGISTRY_METADATA_URL)
            .call()
            .map_err(|error| format!("registry request failed: {error}"))?;
        response
            .body_mut()
            .with_config()
            .limit(MAX_METADATA_BYTES)
            .read_to_vec()
            .map_err(|error| format!("registry response could not be read: {error}"))
    }
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments == ["--check", "--help"] {
        print!("{UPDATE_CHECK_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = parse_arguments(arguments).and_then(|arguments| {
        let now = current_unix_time()?;
        check(&arguments, now, &RegistryClient)
    });
    emit_action_result(&result.unwrap_or_else(failure))
}

fn parse_arguments(arguments: &[String]) -> Result<Arguments, String> {
    let mut check = false;
    let mut user_root = None;
    let mut output = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        index += 1;
        match option {
            "--check" if !check => check = true,
            "--user-root" | "--output" => {
                let value = arguments
                    .get(index)
                    .ok_or_else(|| format!("missing value for {option}"))?;
                index += 1;
                let slot = if option == "--user-root" {
                    &mut user_root
                } else {
                    &mut output
                };
                if slot.replace(value.clone()).is_some() {
                    return Err(format!("duplicate update-check option: {option}"));
                }
            }
            _ => {
                return Err(format!(
                    "unknown or duplicate update-check option: {option}"
                ))
            }
        }
    }
    if !check || output.as_deref() != Some("json") {
        return Err("update check requires --check and --output json".to_owned());
    }
    let user_root =
        PathBuf::from(user_root.ok_or_else(|| "update check requires --user-root".to_owned())?);
    if !user_root.is_absolute() {
        return Err("--user-root must be an absolute directory".to_owned());
    }
    Ok(Arguments { user_root })
}

#[allow(clippy::too_many_lines)]
fn check(
    arguments: &Arguments,
    now_unix: i64,
    client: &impl MetadataClient,
) -> Result<ActionResult, String> {
    let root = super::user_install::open_user_root_for_setup(&arguments.user_root)?;
    let config = super::user_setup::load_operational_config(&root)
        .map_err(|error| error.message().to_owned())?
        .ok_or_else(|| "global Hive setup is required before update checks".to_owned())?;
    if !config.update_check.enabled {
        return Ok(outcome(
            "hive.update-check-disabled",
            "daily update checking is disabled",
            None,
            None,
            false,
            Vec::new(),
            Vec::new(),
        ));
    }

    let relative = Path::new(UPDATE_CHECK_STATE_RELATIVE);
    let existing = super::user_install::read_user_setup_file(&root, relative, MAX_STATE_BYTES)?;
    let state = existing
        .as_deref()
        .and_then(|bytes| serde_json::from_slice::<UpdateCheckState>(bytes).ok())
        .filter(|state| state.schema_version == 1);
    if state.as_ref().is_some_and(|state| {
        now_unix >= state.last_success_unix
            && now_unix - state.last_success_unix < SUCCESS_THROTTLE_SECONDS
    }) {
        let current: SemVersion = env!("CARGO_PKG_VERSION")
            .parse()
            .map_err(|error| format!("compiled Hive version is invalid: {error}"))?;
        let latest = state
            .as_ref()
            .and_then(|state| state.latest_version.parse::<SemVersion>().ok());
        let update_available = latest.is_some_and(|latest| latest > current);
        return Ok(outcome(
            "hive.update-check-throttled",
            "the last successful update check is less than 24 hours old",
            Some(&current.to_string()),
            state.as_ref().map(|state| state.latest_version.as_str()),
            update_available,
            Vec::new(),
            Vec::new(),
        ));
    }

    let metadata = match client.fetch() {
        Ok(metadata) => metadata,
        Err(error) => {
            return Ok(outcome(
                "hive.update-check-deferred",
                &format!("update check deferred until the next host session: {error}"),
                Some(env!("CARGO_PKG_VERSION")),
                None,
                false,
                Vec::new(),
                Vec::new(),
            ));
        }
    };
    let latest = match latest_version(&metadata) {
        Ok(latest) => latest,
        Err(error) => {
            return Ok(outcome(
                "hive.update-check-deferred",
                &format!("update check deferred until the next host session: {error}"),
                Some(env!("CARGO_PKG_VERSION")),
                None,
                false,
                Vec::new(),
                Vec::new(),
            ));
        }
    };
    let current: SemVersion = env!("CARGO_PKG_VERSION")
        .parse()
        .map_err(|error| format!("compiled Hive version is invalid: {error}"))?;
    let update_available = latest > current;
    let state = UpdateCheckState {
        schema_version: 1,
        last_success_unix: now_unix,
        latest_version: latest.to_string(),
    };
    let mut desired = serde_json_canonicalizer::to_vec(&state)
        .map_err(|error| format!("cannot serialize update-check state: {error}"))?;
    desired.push(b'\n');
    super::user_install::replace_user_setup_file(
        &root,
        relative,
        existing.as_deref(),
        Some(&desired),
    )?;
    Ok(outcome(
        if update_available {
            "hive.update-available"
        } else {
            "hive.update-check-current"
        },
        if update_available {
            "a newer Aigent Hive version is available; no update was installed"
        } else {
            "Aigent Hive is current; no update was installed"
        },
        Some(&current.to_string()),
        Some(&latest.to_string()),
        update_available,
        vec![UPDATE_CHECK_STATE_RELATIVE.to_owned()],
        vec![Evidence {
            kind: "report",
            locator: UPDATE_CHECK_STATE_RELATIVE.to_owned(),
            digest: sha256_digest(&desired),
        }],
    ))
}

fn latest_version(bytes: &[u8]) -> Result<SemVersion, String> {
    #[derive(Deserialize)]
    struct RegistryMetadata {
        versions: BTreeMap<String, serde_json::Value>,
    }
    let metadata: RegistryMetadata = serde_json::from_slice(bytes)
        .map_err(|error| format!("registry response is malformed JSON: {error}"))?;
    metadata
        .versions
        .keys()
        .filter_map(|version| version.parse::<SemVersion>().ok())
        .max()
        .ok_or_else(|| "registry response contains no strict released version".to_owned())
}

#[allow(clippy::too_many_arguments)]
fn outcome(
    code: &'static str,
    message: &str,
    current: Option<&str>,
    latest: Option<&str>,
    update_available: bool,
    changed_paths: Vec<String>,
    evidence: Vec<Evidence>,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "CheckHiveUpdate",
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence,
        next_action: update_available.then(|| "run hive update".to_owned()),
        data: Some(json!({
            "current_version": current,
            "latest_version": latest,
            "update_available": update_available,
            "installed": false
        })),
    }
}

fn failure(message: String) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "CheckHiveUpdate",
        status: "error",
        exit_code: 2,
        code: "hive.update-check-invalid-input",
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn current_unix_time() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock precedes Unix epoch".to_owned())?
        .as_secs()
        .try_into()
        .map_err(|_| "system clock is out of range".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    struct FakeClient(Result<Vec<u8>, String>);

    impl MetadataClient for FakeClient {
        fn fetch(&self) -> Result<Vec<u8>, String> {
            self.0.clone()
        }
    }

    fn user_root(enabled: bool) -> tempfile::TempDir {
        let temporary = tempdir().expect("temporary user root");
        let config = temporary.path().join(".hive/config/user-setup.yml");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config directory");
        fs::write(
            config,
            format!(
                "schema_version: 1\ninterface_language: en\nwiki:\n  enabled: true\n  language: en\nprofile:\n  id: non-developer\npersona:\n  id: strict\nselected_hosts:\n  - codex\nskills:\n  mode: individual\n  selected:\n    - setup-hive\nupdate_check:\n  enabled: {enabled}\nusage_guard:\n  enabled: false\n  stop_remaining_percent: 20\n  codexbar_fallback_enabled: false\n"
            ),
        )
        .expect("user config");
        temporary
    }

    #[test]
    fn offline_checks_do_not_record_success_and_retry_next_session() {
        let temporary = user_root(true);
        let arguments = Arguments {
            user_root: temporary.path().to_path_buf(),
        };
        let first =
            check(&arguments, 100, &FakeClient(Err("offline".to_owned()))).expect("deferred check");
        let second = check(
            &arguments,
            101,
            &FakeClient(Ok(br#"{"versions":{"0.7.0":{},"0.8.0":{}}}"#.to_vec())),
        )
        .expect("retried check");

        assert_eq!(first.code, "hive.update-check-deferred");
        assert!(first.changed_paths.is_empty());
        assert_eq!(second.code, "hive.update-available");
        assert_eq!(second.changed_paths, [UPDATE_CHECK_STATE_RELATIVE]);
    }

    #[test]
    fn successful_checks_throttle_for_24_hours_without_network() {
        let temporary = user_root(true);
        let arguments = Arguments {
            user_root: temporary.path().to_path_buf(),
        };
        check(
            &arguments,
            10,
            &FakeClient(Ok(br#"{"versions":{"0.7.0":{}}}"#.to_vec())),
        )
        .expect("first check");
        let throttled = check(
            &arguments,
            10 + SUCCESS_THROTTLE_SECONDS - 1,
            &FakeClient(Err("must not be observed".to_owned())),
        )
        .expect("throttled check");

        assert_eq!(throttled.code, "hive.update-check-throttled");
        assert!(throttled.changed_paths.is_empty());
    }

    #[test]
    fn disabled_checks_never_call_the_registry_or_write_state() {
        let temporary = user_root(false);
        let result = check(
            &Arguments {
                user_root: temporary.path().to_path_buf(),
            },
            100,
            &FakeClient(Err("must not be observed".to_owned())),
        )
        .expect("disabled check");

        assert_eq!(result.code, "hive.update-check-disabled");
        assert!(result.changed_paths.is_empty());
        assert!(!temporary.path().join(UPDATE_CHECK_STATE_RELATIVE).exists());
    }
}
