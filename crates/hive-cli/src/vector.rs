//! Explicitly consented, offline, non-generative vector indexing.

use super::{optional, parse_options, required};
use crate::knowledge_scan::run_bounded_process_with_input;
use hive_core::{normalize_platform_root, sha256_digest};
use hive_wiki::rag::{RagVisibility, SemanticPartition};
use hive_wiki::store::RagStore;
use hive_wiki::vector::VectorFiles;
use hive_wiki::WikiError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;
#[path = "vector_auth.rs"]
mod auth;
#[path = "vector_index.rs"]
mod index;
#[path = "vector_query.rs"]
mod query;

pub(super) fn retrieve(
    root: &Path,
    store: &RagStore,
    request: &hive_wiki::rag::RetrievalRequest,
) -> Result<Value, WikiError> {
    query::retrieve(root, store, request)
}

const BOOTSTRAP: &str = include_str!("vector_runtime.py");
const WORKER: &str = include_str!("vector_helper.py");
const LOCK: &str = include_str!("vector-runtime-lock.json");
const MAX_WORKERS: usize = 16;

#[cfg(test)]
thread_local! {
    static TEST_DEFAULT_WORKERS: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

fn default_workers() -> usize {
    #[cfg(test)]
    if let Some(count) = TEST_DEFAULT_WORKERS.with(std::cell::Cell::get) {
        return count;
    }
    std::thread::available_parallelism().map_or(1, |count| count.get().min(12))
}
const USAGE: &str = "Optional local semantic search; FTS remains the default.\n\
    hive knowledge vector preview|enable|status|rebuild|rollback|disable --user-root <dir> --target <dir> --collection <id> --visibility shared|project-private|confidential [--python <absolute-executable>] [--consent-digest <sha256:...>] --output json\n\
    hive knowledge vector authorize-build --user-root <dir> --target <dir> --collection <id> --visibility confidential --capabilities <json> --usage <json> --expires-at <unix-seconds> --nonce <nonce> --confirm-current-action [--operation rebuild|rollback] --output json\n\
    hive source-wiki vector preview|enable|status|rebuild|rollback|disable|query --target <source-root> --language en|ko [--python <absolute-executable>] [--consent-digest <sha256:...>] --output json\n\
    source query options: --query <text> --top-k <1..100>\n\
    rebuild options: --max-seconds <1..60> --workers <1..16> --rebuild-mode resume|fresh\n\
    confidential rebuild/rollback: --authorization-id <id> --authorization-token <token> --capabilities <json> --usage <json>\n";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Selector {
    Source { language: String },
    Collection { partition: SemanticPartition },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstalledRuntime {
    id: String,
    python: PathBuf,
    identity: Value,
    contract_digest: String,
    receipt_digest: String,
    consent_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeControl {
    schema_version: u32,
    current: InstalledRuntime,
    previous: Option<InstalledRuntime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeControl {
    schema_version: u32,
    #[serde(default)]
    revision: u64,
    selector: Selector,
    enabled: bool,
    consent_digest: String,
    runtime: InstalledRuntime,
    #[serde(default)]
    checkpoint: Option<index::Snapshot>,
    #[serde(default)]
    active: Option<index::Snapshot>,
    #[serde(default)]
    previous: Option<index::Snapshot>,
    #[serde(default)]
    retired: Vec<index::RetiredSnapshot>,
}

struct Target {
    files: VectorFiles,
    selector: Selector,
    scope_id: String,
    current_collection_id: Option<String>,
}

pub(super) fn run(arguments: &[String], source: bool) -> ExitCode {
    if super::is_help(arguments)
        || (arguments.len() == 2
            && arguments
                .last()
                .is_some_and(|value| value == "--help" || value == "-h"))
    {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = dispatch(arguments, source)
        .and_then(|data| {
            let digest = value_digest(&data)?;
            let changed_paths = data
                .get("changed_paths")
                .and_then(Value::as_array)
                .map(|paths| {
                    paths
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default();
            let incomplete = data.get("complete").and_then(Value::as_bool) == Some(false);
            let new_approval = data.get("requires_new_authorization").and_then(Value::as_bool) == Some(true);
            let mut result = super::success(
                "VectorKnowledge",
                "hive.vector-completed",
                "optional vector operation completed",
                changed_paths,
                "vector",
                &digest,
                data,
            );
            if incomplete {
                result.next_action = Some(if new_approval {
                    "issue a new authorize-build current-action approval, then repeat rebuild with its new one-time token"
                } else {
                    "repeat vector rebuild with the same scope to resume its verified checkpoint"
                }.to_owned());
            }
            Ok(result)
        })
        .unwrap_or_else(|error| super::failure("VectorKnowledge", &error));
    super::emit(&result);
    ExitCode::from(result.exit_code)
}

fn dispatch(arguments: &[String], source: bool) -> Result<Value, WikiError> {
    let action = arguments
        .first()
        .ok_or_else(|| invalid("vector action is required"))?;
    let confirmations = arguments
        .iter()
        .filter(|arg| arg.as_str() == "--confirm-current-action")
        .count();
    if confirmations != usize::from(action == "authorize-build") {
        return Err(invalid("authorize-build requires exactly one current-action confirmation; other actions reject it"));
    }
    let filtered = arguments[1..]
        .iter()
        .filter(|arg| arg.as_str() != "--confirm-current-action")
        .cloned()
        .collect::<Vec<_>>();
    let options = parse_options(
        &filtered,
        &[
            "--target",
            "--user-root",
            "--collection",
            "--visibility",
            "--language",
            "--python",
            "--consent-digest",
            "--max-seconds",
            "--workers",
            "--rebuild-mode",
            "--query",
            "--top-k",
            "--authorization-id",
            "--authorization-token",
            "--capabilities",
            "--usage",
            "--expires-at",
            "--nonce",
            "--operation",
        ],
    )?;
    for (name, _) in &options {
        let allowed = match *name {
            "--python" => ["preview", "enable"].contains(&action.as_str()),
            "--consent-digest" => action == "enable",
            "--max-seconds" | "--workers" | "--rebuild-mode" => {
                action == "rebuild" || action == "authorize-build"
            }
            "--query" | "--top-k" => action == "query" && source,
            "--capabilities" | "--usage" => {
                !source && ["authorize-build", "rebuild", "rollback"].contains(&action.as_str())
            }
            "--authorization-id" | "--authorization-token" => {
                !source && ["rebuild", "rollback"].contains(&action.as_str())
            }
            "--expires-at" | "--nonce" | "--operation" => !source && action == "authorize-build",
            _ => true,
        };
        if !allowed {
            return Err(invalid("vector option does not apply to this action"));
        }
    }
    let target = parse_target(&options, source)?;
    match action.as_str() {
        "preview" => preview(&target, &options),
        "enable" => enable(&target, &options),
        "status" => status(&target),
        "disable" => disable(&target),
        "rebuild" => index::rebuild(&target, &options),
        "rollback" => index::rollback(&target, &options),
        "authorize-build" if !source => auth::issue(&target, &options),
        "query" if source => query::source_query(&target, &options),
        _ => Err(invalid("unsupported vector action")),
    }
}

fn parse_target(options: &[(&str, &str)], source: bool) -> Result<Target, WikiError> {
    let target = Path::new(required(options, "--target")?);
    let mut current_collection_id = None;
    let (files, selector) = if source {
        if optional(options, "--user-root").is_some()
            || optional(options, "--collection").is_some()
            || optional(options, "--visibility").is_some()
        {
            return Err(invalid("source vectors cannot use consumer scope options"));
        }
        let language = required(options, "--language")?;
        if !["en", "ko"].contains(&language) {
            return Err(invalid("source vector language must be en or ko"));
        }
        (
            VectorFiles::open(target, true)?,
            Selector::Source {
                language: language.to_owned(),
            },
        )
    } else {
        if optional(options, "--language").is_some() {
            return Err(invalid("consumer vectors use collection language metadata"));
        }
        let root = Path::new(required(options, "--user-root")?);
        super::require_shared_wiki_enabled(root)?;
        let store = RagStore::open(root)?;
        let registry = store.load_registry()?;
        current_collection_id =
            super::derive_optional_current_collection_authority(&registry, target)?.1;
        let id = super::resolve_collection_reference(
            &registry,
            required(options, "--collection")?,
            "vector collection",
        )?;
        let visibility = match required(options, "--visibility")? {
            "shared" => RagVisibility::Shared,
            "project-private" => RagVisibility::ProjectPrivate,
            "confidential" => RagVisibility::Confidential,
            _ => {
                return Err(invalid(
                    "vector visibility requires shared, project-private or confidential",
                ))
            }
        };
        (
            VectorFiles::open(root, false)?,
            Selector::Collection {
                partition: SemanticPartition {
                    collection_id: id,
                    visibility,
                },
            },
        )
    };
    let scope_id = files.scope_id(&selector)?;
    Ok(Target {
        files,
        selector,
        scope_id,
        current_collection_id,
    })
}

fn value_digest(value: &impl Serialize) -> Result<String, WikiError> {
    serde_json_canonicalizer::to_vec(value)
        .map(|bytes| sha256_digest(&bytes))
        .map_err(io_error)
}

fn contract_digest() -> Result<String, WikiError> {
    value_digest(
        &json!({"schema_version":1,"lock":lock()?,"worker":sha256_digest(WORKER.as_bytes()),"bootstrap":sha256_digest(BOOTSTRAP.as_bytes())}),
    )
}

fn lock() -> Result<Value, WikiError> {
    serde_json::from_str(LOCK).map_err(io_error)
}

fn python(options: &[(&str, &str)], files: &VectorFiles) -> Result<PathBuf, WikiError> {
    let path = if let Some(value) = optional(options, "--python") {
        PathBuf::from(value)
    } else {
        let control = runtime_control(files)?.0.ok_or_else(|| invalid("vector preview needs --python with an installed CPython 3.12 or 3.13 executable; Hive never installs Python"))?;
        authenticate_python(&control.current)?;
        control.current.python
    };
    if !path.is_absolute() || !path.is_file() {
        return Err(invalid(
            "vector Python must be an existing absolute executable",
        ));
    }
    Ok(normalize_platform_root(
        &path.canonicalize().map_err(io_error)?,
    ))
}

fn runtime_control(
    files: &VectorFiles,
) -> Result<(Option<RuntimeControl>, Option<String>), WikiError> {
    let (control, digest) = files.read_control::<RuntimeControl>(None)?;
    if control
        .as_ref()
        .is_some_and(|value| value.schema_version != 1)
    {
        return Err(invalid("unsupported vector runtime control"));
    }
    Ok((control, digest))
}

fn scope_control(target: &Target) -> Result<(Option<ScopeControl>, Option<String>), WikiError> {
    let (control, digest) = target
        .files
        .read_control::<ScopeControl>(Some(&target.scope_id))?;
    if control
        .as_ref()
        .is_some_and(|value| value.schema_version != 1 || value.selector != target.selector)
    {
        return Err(invalid("vector scope authority mismatch"));
    }
    Ok((control, digest))
}

fn preview(target: &Target, options: &[(&str, &str)]) -> Result<Value, WikiError> {
    let python = python(options, &target.files)?;
    let identity = bootstrap(
        &python,
        json!({"action":"describe","lock":lock()?}),
        None,
        15,
    )?;
    let mut data = json!({"schema_version":1,"operation":"enable-vector","scope_id":target.scope_id,
        "selector":target.selector,"root":target.files.root_path(),"python":python,"identity":identity,
        "contract_digest":contract_digest()?,"downloads":lock()?,
        "writes_under":[target.files.data_relative(),target.files.control_relative()],
        "operation_directories":["runtimes/<attempt-id>","work/<attempt-id>"],
        "control_files":["runtime.json","runtime.lock","scope-<scope-id>.json","scope-<scope-id>.lock"],
        "provider_api":false,"query_log":false,"python_install":false,
        "complete_pip_environment":false,"fts_unchanged":true});
    data["consent_digest"] = json!(value_digest(&data)?);
    Ok(data)
}

fn enable(target: &Target, options: &[(&str, &str)]) -> Result<Value, WikiError> {
    let expected = required(options, "--consent-digest")?;
    let plan = preview(target, options)?;
    if plan["consent_digest"].as_str() != Some(expected) {
        return Err(invalid(
            "enable requires the exact current preview consent digest",
        ));
    }
    let _runtime_lease = target.files.writer(None)?;
    let _scope_lease = target.files.writer(Some(&target.scope_id))?;
    let (prior, runtime_digest) = runtime_control(&target.files)?;
    let (prior_scope, scope_digest) = scope_control(target)?;
    let python = python(options, &target.files)?;
    let current_contract = contract_digest()?;
    let reusable = prior.as_ref().filter(|control| {
        control.current.contract_digest == current_contract
            && control.current.identity == plan["identity"]
            && verify_runtime(&target.files, &control.current).is_ok()
    });
    let installed = if let Some(control) = reusable {
        verify_runtime(&target.files, &control.current)?;
        control.current.clone()
    } else {
        let (id, root) = target.files.reserve_runtime()?;
        let work = target.files.reserve_work()?;
        check_python(
            &python,
            plan["identity"]["python_digest"]
                .as_str()
                .ok_or_else(|| invalid("Python identity is absent"))?,
        )?;
        let receipt = bootstrap(
            &python,
            json!({"action":"stage","lock":lock()?,"root":root,"cache":work,"helper":WORKER}),
            Some(&work),
            900,
        )?;
        let installed = InstalledRuntime {
            id,
            python,
            identity: plan["identity"].clone(),
            contract_digest: current_contract,
            receipt_digest: receipt["receipt_digest"]
                .as_str()
                .ok_or_else(|| invalid("runtime receipt is absent"))?
                .to_owned(),
            consent_digest: expected.to_owned(),
        };
        verify_runtime(&target.files, &installed)?;
        worker(
            &target.files,
            &installed,
            json!({"schema_version":1,"action":"self-test","runtime":root}),
            30,
        )?;
        installed
    };
    let mut control = prior_scope.unwrap_or(ScopeControl {
        schema_version: 1,
        revision: 0,
        selector: target.selector.clone(),
        enabled: false,
        consent_digest: expected.to_owned(),
        runtime: installed.clone(),
        checkpoint: None,
        active: None,
        previous: None,
        retired: Vec::new(),
    });
    control.enabled = true;
    control.revision = control
        .revision
        .checked_add(1)
        .ok_or_else(|| invalid("vector control revision is exhausted"))?;
    control.runtime = installed.clone();
    expected.clone_into(&mut control.consent_digest);
    if reusable.is_some() {
        target
            .files
            .write_control(Some(&target.scope_id), scope_digest.as_deref(), &control)?;
    } else {
        target.files.write_control_pair(
            (
                runtime_digest.as_deref(),
                &RuntimeControl {
                    schema_version: 1,
                    current: installed.clone(),
                    previous: prior.map(|value| value.current),
                },
            ),
            (&target.scope_id, scope_digest.as_deref(), &control),
        )?;
    }
    Ok(
        json!({"enabled":true,"index_ready":false,"scope_id":target.scope_id,"runtime":installed.id,"fts_unchanged":true,
            "changed_paths":[target.files.data_relative(),target.files.control_relative()]}),
    )
}

fn status(target: &Target) -> Result<Value, WikiError> {
    let (scope, _) = scope_control(target)?;
    let Some(scope) = scope.filter(|control| control.enabled) else {
        return Ok(json!({"enabled":false,"fts_unchanged":true}));
    };
    verify_runtime(&target.files, &scope.runtime)?;
    let mut result = index::status(target, &scope);
    result["enabled"] = json!(true);
    result["runtime_verified"] = json!(true);
    result["fts_unchanged"] = json!(true);
    Ok(result)
}

fn disable(target: &Target) -> Result<Value, WikiError> {
    let _lease = target.files.writer(Some(&target.scope_id))?;
    let (control, digest) = scope_control(target)?;
    if let Some(mut control) = control {
        control.enabled = false;
        control.revision = control
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("vector control revision is exhausted"))?;
        target
            .files
            .write_control(Some(&target.scope_id), digest.as_deref(), &control)?;
    }
    Ok(
        json!({"enabled":false,"derived_files_retained":true,"fts_unchanged":true,"changed_paths":[target.files.control_relative()]}),
    )
}

fn verify_runtime(files: &VectorFiles, runtime: &InstalledRuntime) -> Result<(), WikiError> {
    authenticate_python(runtime)?;
    if runtime.contract_digest != contract_digest()? {
        return Err(invalid(
            "vector runtime contract changed; new preview and approval required",
        ));
    }
    let receipt = bootstrap(
        &runtime.python,
        json!({"action":"verify","root":files.runtime_path(&runtime.id)?,"lock":lock()?,"helper_digest":sha256_digest(WORKER.as_bytes())}),
        None,
        30,
    )?;
    if receipt["receipt_digest"].as_str() != Some(&runtime.receipt_digest)
        || receipt["identity"] != runtime.identity
    {
        return Err(invalid(
            "vector runtime no longer matches its approved receipt",
        ));
    }
    Ok(())
}

fn bootstrap(
    python: &Path,
    request: Value,
    work: Option<&Path>,
    timeout: u64,
) -> Result<Value, WikiError> {
    invoke(
        python,
        &[
            "-I".into(),
            "-S".into(),
            "-B".into(),
            "-c".into(),
            BOOTSTRAP.into(),
        ],
        request,
        work,
        timeout,
    )
}

fn worker(
    files: &VectorFiles,
    runtime: &InstalledRuntime,
    request: Value,
    timeout: u64,
) -> Result<Value, WikiError> {
    authenticate_python(runtime)?;
    let root = files.runtime_path(&runtime.id)?;
    let helper = root.join("vector_helper.py");
    invoke(
        &runtime.python,
        &[
            "-I".into(),
            "-S".into(),
            "-B".into(),
            helper.into_os_string(),
        ],
        request,
        Some(&root.join("tmp")),
        timeout,
    )
}

fn invoke(
    python: &Path,
    arguments: &[OsString],
    request: Value,
    work: Option<&Path>,
    timeout: u64,
) -> Result<Value, WikiError> {
    let mut environment = Vec::new();
    // Only operating-system bootstrap values, never PATH, HOME, host settings or provider keys.
    for name in ["SYSTEMROOT", "WINDIR", "COMSPEC", "LANG", "LC_ALL"] {
        if let Some(value) = std::env::var_os(name) {
            environment.push((OsString::from(name), value));
        }
    }
    if let Some(work) = work {
        for key in ["TEMP", "TMP", "TMPDIR"] {
            environment.push((OsString::from(key), work.as_os_str().to_owned()));
        }
    }
    let input = serde_json::to_vec(&request).map_err(io_error)?;
    // Build requests can be large: release the structured copy before the worker starts.
    drop(request);
    let result = run_bounded_process_with_input(
        python,
        arguments,
        &environment,
        Duration::from_secs(timeout),
        4 * 1024 * 1024,
        Some(input),
        "vector helper",
    )?;
    let value: Value = serde_json::from_slice(&result.stdout)
        .map_err(|_| invalid("vector helper returned invalid JSON"))?;
    if !result.success || value["status"] != "success" {
        let reason = value["error_type"]
            .as_str()
            .filter(|name| {
                [
                    "ValueError",
                    "RuntimeError",
                    "ImportError",
                    "ModuleNotFoundError",
                    "OperationalError",
                    "FileNotFoundError",
                    "PermissionError",
                    "KeyError",
                    "TypeError",
                ]
                .contains(name)
            })
            .unwrap_or("unknown");
        return Err(invalid(&format!(
            "vector helper validation failed ({reason}); FTS remains available"
        )));
    }
    let mut value = value;
    value
        .as_object_mut()
        .ok_or_else(|| invalid("vector helper returned a non-object"))?
        .remove("status");
    Ok(value)
}

fn authenticate_python(runtime: &InstalledRuntime) -> Result<(), WikiError> {
    check_python(
        &runtime.python,
        runtime.identity["python_digest"]
            .as_str()
            .ok_or_else(|| invalid("stored Python digest is absent"))?,
    )
}

fn check_python(path: &Path, expected: &str) -> Result<(), WikiError> {
    use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
    use cap_std::{
        ambient_authority,
        fs::{Dir, OpenOptions},
    };
    use std::io::Read;
    let parent = path
        .parent()
        .ok_or_else(|| invalid("Python has no parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| invalid("Python has no filename"))?;
    if !path.is_absolute() {
        return Err(invalid("stored Python path must be absolute"));
    }
    hive_core::ensure_no_symlink_ancestors(parent, Path::new(name)).map_err(io_error)?;
    let directory = Dir::open_ambient_dir(parent, ambient_authority()).map_err(io_error)?;
    let file = directory
        .open_with(
            name,
            OpenOptions::new().read(true).follow(FollowSymlinks::No),
        )
        .map_err(io_error)?;
    let metadata = file.metadata().map_err(io_error)?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 * 1024 {
        return Err(invalid("Python is not a bounded regular executable"));
    }
    let mut bytes = Vec::new();
    file.take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 != metadata.len() || expected != sha256_digest(&bytes) {
        return Err(invalid("Python changed since approval; refusing execution"));
    }
    Ok(())
}

fn invalid(message: &str) -> WikiError {
    WikiError::InvalidInput(message.to_owned())
}
fn io_error(error: impl std::fmt::Display) -> WikiError {
    WikiError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_parallelism_is_bounded_by_host_capacity() {
        let count = default_workers();
        assert!((1..=12).contains(&count));
        assert!(count <= std::thread::available_parallelism().map_or(1, std::num::NonZero::get));
        assert_eq!(MAX_WORKERS, 16);
    }

    #[test]
    fn source_status_does_not_install_or_create_consumer_state() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        fs::create_dir_all(&base).expect("work");
        let root = tempfile::tempdir_in(base).expect("root");
        fs::write(root.path().join("hive-source.json"), br#"{"schema_version":1,"kind":"aigent-hive-source-workspace","consumer_setup_allowed":false}"#).expect("marker");
        for language in ["en", "ko"] {
            fs::create_dir_all(root.path().join("docs/facts").join(language)).expect("facts");
        }
        let args = vec![
            "status".to_owned(),
            "--target".to_owned(),
            root.path().to_string_lossy().into_owned(),
            "--language".to_owned(),
            "ko".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        assert_eq!(dispatch(&args, true).expect("status")["enabled"], false);
        assert!(!root.path().join(".hive").exists());
        assert!(!root.path().join(".agents").exists());
    }

    #[test]
    fn contract_binds_model_dependencies_bootstrap_and_worker() {
        assert!(contract_digest().expect("contract").starts_with("sha256:"));
        let lock = lock().expect("lock");
        assert_eq!(lock["model"]["dimension"], 384);
        assert_eq!(lock["model"]["token_limit"], 128);
        assert_ne!(
            sha256_digest(WORKER.as_bytes()),
            sha256_digest(BOOTSTRAP.as_bytes())
        );
    }

    #[test]
    fn corrupt_fts_does_not_block_disabling_vectors() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        let root = tempfile::tempdir_in(base).expect("root");
        fs::create_dir_all(root.path().join(".hive/config")).expect("config");
        fs::write(root.path().join(".hive/config/user-setup.yml"), serde_json::to_vec(&json!({
            "schema_version":1,"interface_language":"en","wiki":{"enabled":true,"language":"both"},
            "profile":{"id":"web-developer"},"persona":{"id":"balanced"},"selected_hosts":["codex"],
            "skills":{"mode":"individual","selected":["setup-hive"]},
            "usage_guard":{"enabled":false,"stop_remaining_percent":20,"codexbar_fallback_enabled":false}
        })).expect("preferences")).expect("config");
        RagStore::open(root.path())
            .expect("store")
            .ensure_registry()
            .expect("registry");
        fs::write(
            root.path().join(hive_wiki::shared::SHARED_INDEX_RELATIVE),
            b"corrupt SQLite",
        )
        .expect("corrupt index");
        let root_arg =
            normalize_platform_root(&root.path().canonicalize().expect("canonical root"))
                .to_string_lossy()
                .into_owned();
        let args = vec![
            "disable".to_owned(),
            "--target".to_owned(),
            root_arg.clone(),
            "--user-root".to_owned(),
            root_arg,
            "--collection".to_owned(),
            "user-root".to_owned(),
            "--visibility".to_owned(),
            "shared".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        assert_eq!(
            dispatch(&args, false).expect("disable without index")["enabled"],
            false
        );
        assert_eq!(
            fs::read(root.path().join(hive_wiki::shared::SHARED_INDEX_RELATIVE))
                .expect("preserved"),
            b"corrupt SQLite"
        );
    }

    #[test]
    fn changed_python_is_rejected_before_running_it() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/work");
        let root = tempfile::tempdir_in(base).expect("root");
        let executable = root.path().join("python.exe");
        fs::write(&executable, b"approved synthetic executable").expect("fixture");
        let digest = sha256_digest(b"approved synthetic executable");
        check_python(&executable, &digest).expect("approved bytes");
        fs::write(&executable, b"changed executable must never run").expect("replacement");
        assert!(check_python(&executable, &digest).is_err());
        assert!(check_python(Path::new("relative-python"), &digest).is_err());
    }
}
