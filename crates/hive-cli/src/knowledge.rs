use crate::knowledge_scan::scan_directory;
#[path = "knowledge_transfer.rs"]
mod transfer;
#[path = "vector.rs"]
mod vector;
use cap_fs_ext::{FollowSymlinks, MetadataExt as CapMetadataExt, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};
use hive_core::{ensure_consumer_target, ensure_no_symlink_ancestors, sha256_digest};
use hive_wiki::bundle_io::BundlePublishMode;
use hive_wiki::bundle_store::{
    export_bundle, import_bundle, import_bundle_reviewed, preview_export_bundle,
    BundleExportDisposition, BundleImportApproval, BundleImportMode, BundleImportResult,
};
use hive_wiki::collection::{
    CollectionKind, CollectionResolution, CollectionState, CollectionVisibility,
    USER_ROOT_COLLECTION_ID,
};
#[cfg(feature = "notion-preview")]
use hive_wiki::notion::{
    retrieve_persisted as retrieve_notion_persisted, sync_and_publish as sync_notion_and_publish,
    validate_write_receipt, NotionCapabilityReceipt, NotionPersistedOutcome, NotionSyncRequest,
    NotionWriteReceipt, NOTION_LEDGER_RELATIVE,
};
use hive_wiki::portable::{BundleLimits, BundleScope};
use hive_wiki::rag::{
    plan_remember, AssertionStatus, CanonicalClaim, ClaimKind, ClaimProvenance, RagError,
    RagVisibility, RememberRequest, RememberSourceKind, RetrievalRequest, RetrievalScope,
};
use hive_wiki::scan::{validate_claims, ReviewedClaim, ScanInventory};
use hive_wiki::shared::{
    ensure_project_registry, load_project_registry, query_shared_filtered, rebuild_shared_index,
    validate_shared_index, SHARED_INDEX_RELATIVE,
};
use hive_wiki::store::{
    validate_reviewed_claims_for_apply, CollectionRegistration, RagStore,
    SharedKnowledgeOperationLock, StoreCommit,
};
use hive_wiki::{
    activate_generation, build_graph as build_consumer_graph, delete_page, delete_page_shared,
    ensure_graph_owned_path, export_generation, generation_relative_path, ingest, ingest_shared,
    lint, list_pages, load_active_generation, normalize_graphify_code, promote, promote_shared,
    query_filtered, query_generation, query_node_metadata, read_page, rebuild_index,
    remove_active_generation, suppress, suppress_shared, LintIssue, LintSeverity,
    PromotionCategory, PromotionMode, SuppressionEntry, WikiError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;

const KNOWLEDGE_USAGE: &str = "\
Canonical Markdown knowledge and disposable SQLite index.

USAGE:
    hive knowledge ingest --target <dir> --source <file> --wiki <draft.md> [--user-root <dir>] --output json
    hive knowledge add --target <dir> --source <file> --wiki <draft.md> [--quick] [--user-root <dir>] --output json
    hive knowledge query --target <dir> (--text <query>|--tag <tag>|--category <category>) [--limit <1..100>] [--user-root <dir>] --output json
    hive knowledge list --target <dir> [--tag <tag>] [--category <category>] [--limit <1..100>] [--user-root <dir>] --output json
    hive knowledge read --target <dir> --page-id <id> [--user-root <dir>] --output json
    hive knowledge promote --target <project> --user-root <dir> --page-id <id> --category fact|preference|workflow (--dry-run|--apply) --output json
    hive knowledge promote --user-root <dir> --collection <id-or-alias> [--review-id <id>] --dry-run --output json
    hive knowledge promote --user-root <dir> --collection <id-or-alias> --review-id <id> --expected-source-digest <sha256:...> --confirm-global-promotion --apply --output json
    hive knowledge lint --target <dir> [--user-root <dir>] --output json
    hive knowledge delete --target <dir> --page-id <id> --reason <text> [--replacement <locator>] --timestamp <RFC3339> [--user-root <dir>] --output json
    hive knowledge suppress --target <dir> --fingerprint <sha256:...> --source-locator <locator> --reason <text> [--replacement <locator>] --timestamp <RFC3339> [--user-root <dir>] --output json
    hive knowledge remember --user-root <dir> (--request <request.json>|--user-statement <normalized-fact> --claim-key <stable-key> [--kind project-profile|decision|convention|preference|workflow]) --output json
    hive knowledge retrieve --user-root <dir> --target <current-dir> (--request <request.json>|--query <text> [--scope <scope>] [--top-k <1..100>] [--byte-budget <bytes>]) [--mode fts|semantic] [--authorization-id <id> --authorization-token <token> --capabilities <json> --usage <json>] --output json
    hive knowledge authorize-confidential --user-root <dir> --target <current-dir> --collection <id-or-alias> --query <text> --capabilities <json> --usage <json> --expires-at <unix-seconds> --nonce <nonce> --confirm-current-action --output json
    hive knowledge authorize-collection --user-root <dir> --operation attach|map|detach --collection <id-or-alias> [--target <dir>] --expires-at <unix-seconds> --nonce <nonce> --confirm-current-action --output json
    hive knowledge collection attach|map --user-root <dir> --collection <id-or-alias> --target <dir> --authorization-id <id> --authorization-token <token> --output json
    hive knowledge collection detach --user-root <dir> --collection <id-or-alias> --authorization-id <id> --authorization-token <token> --output json
    hive knowledge scan --target <dir> (--inventory|--candidates <review.json>|--apply <review.json>) [--include-untracked] [--prior-inventory <json>] [--user-root <dir>] --output json
    hive knowledge export --user-root <dir> --scope global|shared|project:<id>|collection:<id>|all-portable --bundle <path>.hivekb [--replace-backup <file-name>] --output json
    hive knowledge import --user-root <dir> --bundle <path>.hivekb (--dry-run|--apply) --output json
    hive knowledge transfer export --preview|--apply --user-root <dir> --scope global|shared|project:<id>|collection:<id>|all-portable --bundle <path>.hivekb [--replace-backup <file-name>] --output json
    hive knowledge transfer import --preview|--apply [--exclude-conflicts] [--preview-digest <sha256:...> --expected-sha256 <sha256:...>] --user-root <dir> --bundle <path>.hivekb --output json
    hive knowledge refresh (--target <legacy-project>|--user-root <dir>) --output json
    hive knowledge graph preview|enable|status|rebuild|disable|query|export --target <dir> [--scope project] [--engine native-markdown|graphify-code] [--consent-digest <sha256:...>] [--input <graph.json> --receipt <receipt.json>] [--node-id <id>] [--text <query>] [--user-root <dir>] [--format json|html] --output json
    hive knowledge vector --help
    hive index rebuild (--target <legacy-project>|--user-root <dir>) --output json
";

#[cfg(feature = "notion-preview")]
const NOTION_KNOWLEDGE_USAGE: &str = "\
    hive knowledge notion sync --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> --output json
    hive knowledge notion rebuild --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> --output json
    hive knowledge notion retrieve --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> (--request <request.json>|--query <text> [--scope global] [--top-k <1..100>] [--byte-budget <bytes>]) --output json
    hive knowledge notion write-through --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> --write-receipt <confirmed-write.json> --output json
";

const LEGACY_DERIVED_RELATIVES: [&str; 4] = [
    ".hive/index/hive.sqlite3",
    ".hive/index/hive.sqlite3-wal",
    ".hive/index/hive.sqlite3-shm",
    ".hive/index/.stale",
];

const GRAPHIFY_VERSION: &str = "0.9.47";
const GRAPHIFY_WHEEL_DIGEST: &str =
    "sha256:2a8b13ccd53d507d16dcc12aebe488517c369afa547938464474fd3e772938ab";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const GRAPHIFY_DEPENDENCY_LOCK: &[u8] =
    include_bytes!("../../../harness/dependencies/graphify/0.9.47/windows-x64.json");
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
const GRAPHIFY_DEPENDENCY_LOCK: &[u8] =
    include_bytes!("../../../harness/dependencies/graphify/0.9.47/linux-musl-x64.json");
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const GRAPHIFY_DEPENDENCY_LOCK: &[u8] =
    include_bytes!("../../../harness/dependencies/graphify/0.9.47/macos-arm64.json");
#[cfg(not(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "x86_64", target_env = "musl"),
    all(target_os = "macos", target_arch = "aarch64")
)))]
const GRAPHIFY_DEPENDENCY_LOCK: &[u8] = b"";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphifyCodeReceipt {
    schema_version: u32,
    package_version: String,
    wheel_digest: String,
    dependency_lock_digest: String,
    executable_digest: String,
    python_identity_digest: String,
    consent_digest: String,
    source_commit: String,
    source_tree_digest: String,
    graph_input_digest: String,
    command: Vec<String>,
    provider_api_calls: u32,
    api_keys_read: u32,
    query_logs: u32,
    watcher: bool,
    git_hooks: bool,
    mcp_registration: bool,
    network_requests: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GraphifyConsent {
    schema_version: u32,
    scope: String,
    package_version: String,
    wheel_digest: String,
    dependency_lock_digest: String,
    command: Vec<String>,
    consent_digest: String,
}

const PROJECT_GRAPHIFY_CONSENT_RELATIVE: &str = ".hive/config/graphify-code-consent.json";
const SOURCE_GRAPHIFY_CONSENT_RELATIVE: &str = ".agents/work/graphify/graphify-code-consent.json";

#[cfg(feature = "notion-preview")]
type NotionSyncInputs<'a> = (
    PathBuf,
    NotionCapabilityReceipt,
    NotionSyncRequest,
    Vec<(&'a str, &'a str)>,
);

const KNOWLEDGE_AUTHORIZATION_RELATIVE: &str = ".hive/runtime/knowledge-authorizations";
const KNOWLEDGE_AUTHORIZATION_CONSUMED_RELATIVE: &str =
    ".hive/runtime/knowledge-authorizations/consumed";
const KNOWLEDGE_AUTHORIZATION_BINDINGS_RELATIVE: &str =
    ".hive/runtime/knowledge-authorizations/bindings";
const MAX_AUTHORIZATION_BYTES: usize = 64 * 1024;
const MAX_AUTHORIZATION_RECORDS: usize = 256;
const MAX_AUTHORIZATION_TTL_SECONDS: u64 = 60;
const MAX_AUTHORIZATION_SNAPSHOT_BYTES: u64 = 1024 * 1024;

#[derive(Serialize)]
struct KnowledgeResult {
    schema_version: u32,
    action: &'static str,
    status: &'static str,
    exit_code: u8,
    code: &'static str,
    message: String,
    changed_paths: Vec<String>,
    evidence: Vec<KnowledgeEvidence>,
    next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Serialize)]
struct KnowledgeEvidence {
    kind: &'static str,
    locator: String,
    digest: String,
}

pub(crate) fn run_knowledge(arguments: &[String]) -> ExitCode {
    if arguments.first().map(String::as_str) == Some("vector") {
        return vector::run(&arguments[1..], false);
    }
    if is_help(arguments) {
        print!("{KNOWLEDGE_USAGE}");
        #[cfg(feature = "notion-preview")]
        print!("{NOTION_KNOWLEDGE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result =
        match arguments.first().map(String::as_str) {
            Some("ingest") => run_ingest(&arguments[1..])
                .unwrap_or_else(|error| failure("IngestKnowledge", &error)),
            Some("add") => {
                run_add(&arguments[1..]).unwrap_or_else(|error| failure("AddKnowledge", &error))
            }
            Some("query") => {
                run_query(&arguments[1..]).unwrap_or_else(|error| failure("QueryKnowledge", &error))
            }
            Some("graph") => {
                run_graph(&arguments[1..]).unwrap_or_else(|error| failure("QueryKnowledge", &error))
            }
            Some("list") => {
                run_list(&arguments[1..]).unwrap_or_else(|error| failure("ListKnowledge", &error))
            }
            Some("read") => {
                run_read(&arguments[1..]).unwrap_or_else(|error| failure("ReadKnowledge", &error))
            }
            Some("promote") => run_promote(&arguments[1..])
                .unwrap_or_else(|error| failure("PromoteKnowledge", &error)),
            Some("lint") => {
                run_lint(&arguments[1..]).unwrap_or_else(|error| failure("LintKnowledge", &error))
            }
            Some("delete") => run_delete(&arguments[1..])
                .unwrap_or_else(|error| failure("DeleteKnowledge", &error)),
            Some("suppress") => run_suppress(&arguments[1..])
                .unwrap_or_else(|error| failure("SuppressKnowledge", &error)),
            Some("remember") => run_remember(&arguments[1..])
                .unwrap_or_else(|error| failure("RememberKnowledge", &error)),
            Some("retrieve") => run_retrieve(&arguments[1..])
                .unwrap_or_else(|error| failure("RetrieveKnowledge", &error)),
            #[cfg(feature = "notion-preview")]
            Some("notion") => run_notion(&arguments[1..])
                .unwrap_or_else(|error| failure("NotionKnowledge", &error)),
            Some("authorize-confidential") => run_authorize_confidential(&arguments[1..])
                .unwrap_or_else(|error| failure("AuthorizeConfidentialKnowledge", &error)),
            Some("authorize-collection") => run_authorize_collection(&arguments[1..])
                .unwrap_or_else(|error| failure("MapKnowledgeCollection", &error)),
            Some("collection") => run_collection(&arguments[1..])
                .unwrap_or_else(|error| failure("MapKnowledgeCollection", &error)),
            Some("refresh") => run_refresh(&arguments[1..])
                .unwrap_or_else(|error| failure("RefreshKnowledge", &error)),
            Some("scan") => {
                run_scan(&arguments[1..]).unwrap_or_else(|error| failure("ScanKnowledge", &error))
            }
            Some("export") => run_export(&arguments[1..])
                .unwrap_or_else(|error| failure("ExportKnowledge", &error)),
            Some("import") => run_import(&arguments[1..])
                .unwrap_or_else(|error| failure("ImportKnowledge", &error)),
            Some("transfer") => run_transfer(&arguments[1..])
                .unwrap_or_else(|error| failure("TransferKnowledge", &error)),
            Some(action) => failure(
                "IngestKnowledge",
                &WikiError::InvalidInput(format!("unknown knowledge action: {action}")),
            ),
            None => failure(
                "IngestKnowledge",
                &WikiError::InvalidInput("knowledge requires an action".to_owned()),
            ),
        };
    emit(&result);
    ExitCode::from(result.exit_code)
}

fn run_transfer(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    match arguments.first().map(String::as_str) {
        Some("export") => run_transfer_export(&arguments[1..]),
        Some("import") => run_transfer_import(&arguments[1..]),
        Some("status" | "vector") => transfer::run(arguments),
        _ => Err(WikiError::InvalidInput(
            "knowledge transfer requires export or import".to_owned(),
        )),
    }
}

fn run_transfer_export(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let preview_count = arguments
        .iter()
        .filter(|argument| *argument == "--preview")
        .count();
    let apply_count = arguments
        .iter()
        .filter(|argument| *argument == "--apply")
        .count();
    if preview_count + apply_count != 1 {
        return Err(WikiError::InvalidInput(
            "knowledge transfer export requires exactly one of --preview or --apply".to_owned(),
        ));
    }
    if apply_count == 1 {
        let forwarded = arguments
            .iter()
            .filter(|argument| argument.as_str() != "--apply")
            .cloned()
            .collect::<Vec<_>>();
        return run_export(&forwarded);
    }
    let filtered = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--preview")
        .cloned()
        .collect::<Vec<_>>();
    let options = parse_options(
        &filtered,
        &["--user-root", "--scope", "--bundle", "--replace-backup"],
    )?;
    if optional(&options, "--replace-backup").is_some() {
        return Err(WikiError::InvalidInput(
            "transfer export preview does not replace an existing bundle".to_owned(),
        ));
    }
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let bundle = PathBuf::from(required(&options, "--bundle")?);
    let scope = parse_bundle_scope(required(&options, "--scope")?)?;
    let preview = preview_export_bundle(&user_root, scope, BundleLimits::default())?;
    let digest = preview.archive_sha256.clone();
    Ok(success(
        "ExportKnowledge",
        "hive.knowledge-transfer-export-previewed",
        "portable canonical knowledge export preview completed without bundle write",
        Vec::new(),
        &bundle.display().to_string(),
        &digest,
        serde_json::to_value(preview).map_err(|error| WikiError::Io(error.to_string()))?,
    ))
}

fn run_transfer_import(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let preview_count = arguments
        .iter()
        .filter(|argument| *argument == "--preview")
        .count();
    let apply_count = arguments
        .iter()
        .filter(|argument| *argument == "--apply")
        .count();
    let exclude_count = arguments
        .iter()
        .filter(|argument| *argument == "--exclude-conflicts")
        .count();
    if preview_count + apply_count != 1 || exclude_count > 1 {
        return Err(WikiError::InvalidInput(
            "knowledge transfer import requires exactly one of --preview or --apply".to_owned(),
        ));
    }
    let filtered = arguments
        .iter()
        .filter(|arg| !["--preview", "--apply", "--exclude-conflicts"].contains(&arg.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let options = parse_options(
        &filtered,
        &[
            "--user-root",
            "--bundle",
            "--preview-digest",
            "--expected-sha256",
        ],
    )?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let bundle = PathBuf::from(required(&options, "--bundle")?);
    let preview_digest = optional(&options, "--preview-digest");
    let expected_archive = optional(&options, "--expected-sha256");
    if preview_count == 1 {
        if preview_digest.is_some() || exclude_count != 0 {
            return Err(WikiError::InvalidInput(
                "transfer import preview does not accept --preview-digest".to_owned(),
            ));
        }
        let mut result = run_import_mode(&user_root, &bundle, BundleImportMode::DryRun)?;
        if let Some(expected) = expected_archive {
            if result
                .data
                .as_ref()
                .and_then(|data| data.get("archive_sha256"))
                .and_then(Value::as_str)
                != Some(expected)
            {
                return Err(WikiError::Conflict(
                    "bundle SHA-256 differs from the sending computer".to_owned(),
                ));
            }
        }
        let digest = transfer_preview_digest(&result)?;
        insert_transfer_preview_digest(&mut result, &digest)?;
        return Ok(result);
    }
    let expected = preview_digest.ok_or_else(|| {
        WikiError::InvalidInput("transfer import apply requires --preview-digest".to_owned())
    })?;
    let expected_archive = expected_archive.ok_or_else(|| {
        WikiError::InvalidInput("transfer import apply requires --expected-sha256".to_owned())
    })?;
    if !valid_sha256_digest(expected) || !valid_sha256_digest(expected_archive) {
        return Err(WikiError::InvalidInput(
            "transfer import preview digest must be sha256".to_owned(),
        ));
    }
    let imported = import_bundle_reviewed(
        &user_root,
        &bundle,
        if exclude_count == 1 {
            BundleImportMode::ApplyExcludingConflicts
        } else {
            BundleImportMode::Apply
        },
        BundleLimits::default(),
        BundleImportApproval {
            archive_sha256: expected_archive,
            preview_digest: expected,
        },
    )?;
    let mut applied = import_report(&bundle, imported)?;
    transfer::finish_import(&user_root, &mut applied);
    Ok(applied)
}

fn transfer_preview_digest(result: &KnowledgeResult) -> Result<String, WikiError> {
    let data = result.data.as_ref().ok_or_else(|| {
        WikiError::Verification("bundle import preview returned no result data".to_owned())
    })?;
    serde_json::to_vec(data)
        .map(|bytes| sha256_digest(&bytes))
        .map_err(|error| WikiError::Io(format!("cannot digest transfer preview: {error}")))
}

fn insert_transfer_preview_digest(
    result: &mut KnowledgeResult,
    preview_digest: &str,
) -> Result<(), WikiError> {
    let data = result.data.as_mut().ok_or_else(|| {
        WikiError::Verification("bundle import preview returned no result data".to_owned())
    })?;
    data.as_object_mut()
        .ok_or_else(|| WikiError::Verification("bundle import result is not an object".to_owned()))?
        .insert(
            "transfer_preview_digest".to_owned(),
            Value::String(preview_digest.to_owned()),
        );
    Ok(())
}

pub(crate) fn run_source_vector(arguments: &[String]) -> ExitCode {
    vector::run(arguments, true)
}

#[allow(clippy::too_many_lines)]
fn run_graph(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let action = arguments.first().map(String::as_str).ok_or_else(|| {
        WikiError::InvalidInput(
            "knowledge graph requires preview, enable, status, rebuild, disable, query, or export"
                .to_owned(),
        )
    })?;
    if !matches!(
        action,
        "preview" | "enable" | "status" | "rebuild" | "disable" | "query" | "export"
    ) {
        return Err(WikiError::InvalidInput(
            "knowledge graph supports preview, enable, status, rebuild, disable, query, or export"
                .to_owned(),
        ));
    }
    let options = parse_options(
        &arguments[1..],
        &[
            "--target",
            "--scope",
            "--node-id",
            "--engine",
            "--input",
            "--receipt",
            "--format",
            "--text",
            "--user-root",
            "--consent-digest",
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let scope = optional(&options, "--scope").unwrap_or("project");
    if !matches!(scope, "project" | "source") {
        return Err(WikiError::InvalidInput(
            "knowledge graph command supports only project or source scope".to_owned(),
        ));
    }
    let engine = optional(&options, "--engine").unwrap_or("native-markdown");
    if !matches!(engine, "native-markdown" | "graphify-code") {
        return Err(WikiError::InvalidInput(
            "knowledge graph engine is unsupported".to_owned(),
        ));
    }
    if action == "enable" {
        if engine != "graphify-code" {
            return Err(WikiError::InvalidInput(
                "knowledge graph enable is only for graphify-code".to_owned(),
            ));
        }
        let consent = graphify_consent(scope)?;
        if required(&options, "--consent-digest")? != consent.consent_digest {
            return Err(WikiError::Conflict(
                "Graphify enable requires the exact preview consent digest".to_owned(),
            ));
        }
        let changed = persist_graphify_consent(&target, scope, &consent)?;
        let consent_relative = graphify_consent_relative(scope)?;
        return Ok(success(
            "UpdateHarness",
            "hive.knowledge-graph-enabled",
            "Graphify code-only consent enabled without package installation",
            changed,
            consent_relative,
            &consent.consent_digest,
            json!({
                "scope": scope,
                "engine": engine,
                "consent_digest": consent.consent_digest.clone(),
                "package_installed": false,
                "automatic_install": false,
                "writes": true
            }),
        ));
    }
    if engine == "graphify-code" && action == "preview" {
        let graph = build_graph_for_scope(&target, scope)?;
        let consent = graphify_consent(scope)?;
        return Ok(success(
            "QueryKnowledge",
            "hive.knowledge-graph-preview",
            "Graphify code-only full rebuild preview completed without writes",
            Vec::new(),
            "graphify-code-preview",
            &graph.generation_digest,
            json!({
                "scope": scope,
                "engine": engine,
                "package_version": GRAPHIFY_VERSION,
                "wheel_digest": GRAPHIFY_WHEEL_DIGEST,
                "dependency_lock_digest": graphify_dependency_lock_digest()?,
                "command": ["extract", "--force", "--code-only", "--no-cluster"],
                "provider_api_calls": 0,
                "api_keys_read": 0,
                "query_logs": 0,
                "watcher": false,
                "git_hooks": false,
                "mcp_registration": false,
                "network_requests": 0,
                "consent_digest": consent.consent_digest,
                "automatic_install": false,
                "writes": false,
            }),
        ));
    }

    let (graph, source_digest, receipt_digest, fallback, active) =
        if engine == "graphify-code" && action == "rebuild" {
            let input_path = PathBuf::from(required(&options, "--input")?);
            let receipt_path = PathBuf::from(required(&options, "--receipt")?);
            let input = read_bytes_bounded(&input_path, "Graphify graph input", 128 * 1024 * 1024)?;
            let receipt_bytes = read_bytes_bounded(&receipt_path, "Graphify receipt", 1024 * 1024)?;
            let receipt: GraphifyCodeReceipt =
                serde_json::from_slice(&receipt_bytes).map_err(|error| {
                    WikiError::InvalidInput(format!("invalid Graphify receipt: {error}"))
                })?;
            let consent = load_graphify_consent(&target, scope)?;
            validate_graphify_receipt(&receipt, &input, &consent.consent_digest)?;
            (
                normalize_graphify_code(scope, &input).map_err(WikiError::Verification)?,
                receipt.source_tree_digest,
                Some(sha256_digest(&receipt_bytes)),
                false,
                true,
            )
        } else if engine == "graphify-code" {
            match load_active_generation(&target, scope, engine) {
                Ok((pointer, graph)) => (
                    graph,
                    pointer.source_digest,
                    pointer.extractor_receipt_digest,
                    false,
                    true,
                ),
                Err(_) if matches!(action, "query" | "status" | "disable") => {
                    let graph = build_graph_for_scope(&target, scope)?;
                    let source_digest = graph.generation_digest.clone();
                    (graph, source_digest, None, true, false)
                }
                Err(error) => return Err(WikiError::Verification(error)),
            }
        } else {
            let graph = build_graph_for_scope(&target, scope)?;
            let source_digest = graph.generation_digest.clone();
            let active = load_active_generation(&target, scope, engine)
                .is_ok_and(|(pointer, _)| pointer.source_digest == source_digest);
            (graph, source_digest, None, false, active)
        };
    let locator = generation_relative_path(scope, &graph.generation_digest)
        .map_err(WikiError::InvalidInput)?
        .to_string_lossy()
        .replace('\\', "/");
    let mut changed_paths = if action == "rebuild" {
        activate_generation(&target, &graph, &source_digest, receipt_digest.as_deref())
            .map_err(WikiError::Io)?
            .into_iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect()
    } else if action == "disable" {
        remove_active_generation(&target, scope, engine)
            .map_err(WikiError::Io)?
            .into_iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect()
    } else if action == "export" {
        vec![
            export_generation(&target, &graph, required(&options, "--format")?)
                .map_err(WikiError::Io)?
                .to_string_lossy()
                .replace('\\', "/"),
        ]
    } else {
        Vec::new()
    };
    if action == "disable" && engine == "graphify-code" && remove_graphify_consent(&target, scope)?
    {
        changed_paths.push(graphify_consent_relative(scope)?.to_owned());
        changed_paths.sort();
    }
    let data = if action == "query" {
        let node_id = optional(&options, "--node-id");
        let text = optional(&options, "--text");
        if node_id.is_none() && text.is_none() {
            return Err(WikiError::InvalidInput(
                "knowledge graph query requires --node-id or --text".to_owned(),
            ));
        }
        let mut graph_matches = node_id
            .map(|id| query_generation(&graph, id, 50))
            .unwrap_or_default();
        let mut metadata = node_id
            .map(|id| query_node_metadata(&graph, id, 10))
            .unwrap_or_default();
        let fts = if let Some(query) = text {
            if scope == "source" {
                let planned = relation_question_subject(query).unwrap_or(query);
                let hits = hive_wiki::source::query(&target, "en", Some(planned), None, 10)?;
                Some(json!({"count": hits.len(), "hits": hits, "language": "en"}))
            } else {
                let mut query_arguments = vec![
                    "--target".to_owned(),
                    target.to_string_lossy().into_owned(),
                    "--text".to_owned(),
                    query.to_owned(),
                    "--limit".to_owned(),
                    "10".to_owned(),
                    "--output".to_owned(),
                    "json".to_owned(),
                ];
                if let Some(user_root) = optional(&options, "--user-root") {
                    query_arguments.extend(["--user-root".to_owned(), user_root.to_owned()]);
                }
                run_query(&query_arguments)?.data
            }
        } else {
            None
        };
        if node_id.is_none() {
            if let Some(value) = &fts {
                for id in fts_hit_ids(value) {
                    graph_matches.extend(query_generation(&graph, &id, 50));
                    metadata.extend(query_node_metadata(&graph, &id, 10));
                }
                graph_matches.sort();
                graph_matches.dedup();
                graph_matches.truncate(50);
                metadata.sort();
                metadata.dedup();
                metadata.truncate(10);
            }
        }
        let mut matched_lanes = Vec::new();
        if text.is_some() {
            matched_lanes.push("fts");
        }
        if node_id.is_some() || !graph_matches.is_empty() {
            matched_lanes.push(if graph.engine == "graphify-code" {
                "code-graph"
            } else {
                "markdown-graph"
            });
        }
        json!({
            "scope": graph.scope,
            "engine": graph.engine,
            "requested_engine": engine,
            "fallback": fallback,
            "generation_digest": graph.generation_digest,
            "node_id": node_id,
            "text": text,
            "matched_lanes": matched_lanes,
            "fts": fts,
            "matches": graph_matches,
            "metadata": metadata,
            "read_command": node_id.map(|id| format!("hive knowledge read --target <dir> --page-id {id} --output json")),
            "cost_receipt": {
                "nodes_scanned": graph.nodes.len(),
                "edges_scanned": graph.edges.len(),
                "maximum_results": 50,
                "metadata_limit": 10,
                "body_bytes": 0
            },
            "writes": false,
        })
    } else {
        json!({
            "scope": graph.scope,
            "engine": graph.engine,
            "requested_engine": engine,
            "fallback": fallback,
            "generation_digest": graph.generation_digest,
            "node_count": graph.nodes.len(),
            "edge_count": graph.edges.len(),
            "generation_path": locator,
            "active": active || action == "rebuild",
            "writes": matches!(action, "rebuild" | "disable" | "export"),
            "cost_receipt": {
                "nodes": graph.nodes.len(),
                "edges": graph.edges.len(),
                "body_bytes": 0
            },
        })
    };
    Ok(success(
        if action == "rebuild" {
            "RebuildKnowledgeIndex"
        } else if action == "disable" {
            "DeleteKnowledge"
        } else if action == "export" {
            "ExportKnowledge"
        } else {
            "QueryKnowledge"
        },
        match action {
            "rebuild" => "hive.knowledge-graph-rebuilt",
            "disable" => "hive.knowledge-graph-disabled",
            "status" => "hive.knowledge-graph-status",
            "query" => "hive.knowledge-graph-query-complete",
            "export" => "hive.knowledge-graph-exported",
            _ => "hive.knowledge-graph-preview",
        },
        "knowledge graph operation completed",
        changed_paths,
        &locator,
        &graph.generation_digest,
        data,
    ))
}

fn build_graph_for_scope(
    target: &Path,
    scope: &str,
) -> Result<hive_wiki::GraphGeneration, WikiError> {
    if scope == "source" {
        hive_wiki::source::build_graph(target)
    } else {
        build_consumer_graph(target, scope)
    }
}

fn fts_hit_ids(value: &Value) -> Vec<String> {
    value
        .get("hits")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|hit| {
            hit.get("id")
                .or_else(|| hit.get("pair_id"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}

fn relation_question_subject(value: &str) -> Option<&str> {
    value
        .strip_prefix("How does ")
        .and_then(|rest| rest.split_once(" relate to "))
        .map(|(subject, _)| subject.trim())
        .filter(|subject| !subject.is_empty())
}

pub(crate) fn run_source_graph(arguments: &[String]) -> ExitCode {
    if arguments.iter().any(|argument| argument == "--scope") {
        let result = failure(
            "QueryKnowledge",
            &WikiError::InvalidInput("source-wiki graph owns the source scope".to_owned()),
        );
        emit(&result);
        return ExitCode::from(result.exit_code);
    }
    let mut scoped = arguments.to_vec();
    scoped.extend(["--scope".to_owned(), "source".to_owned()]);
    let result = run_graph(&scoped).unwrap_or_else(|error| failure("QueryKnowledge", &error));
    emit(&result);
    ExitCode::from(result.exit_code)
}

fn read_bytes_bounded(path: &Path, label: &str, maximum: u64) -> Result<Vec<u8>, WikiError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| WikiError::Io(format!("cannot inspect {label}: {error}")))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err(WikiError::InvalidInput(format!(
            "{label} must be a nonempty bounded regular file"
        )));
    }
    fs::read(path).map_err(|error| WikiError::Io(format!("cannot read {label}: {error}")))
}

fn validate_graphify_receipt(
    receipt: &GraphifyCodeReceipt,
    input: &[u8],
    consent_digest: &str,
) -> Result<(), WikiError> {
    let digest = |value: &str| {
        value.strip_prefix("sha256:").is_some_and(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    };
    if receipt.schema_version != 1
        || receipt.package_version != GRAPHIFY_VERSION
        || receipt.wheel_digest != GRAPHIFY_WHEEL_DIGEST
        || !digest(&receipt.executable_digest)
        || receipt.dependency_lock_digest != graphify_dependency_lock_digest()?
        || !digest(&receipt.python_identity_digest)
        || receipt.consent_digest != consent_digest
        || !digest(&receipt.source_tree_digest)
        || receipt.source_commit.len() != 40
        || !receipt
            .source_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || receipt.graph_input_digest != sha256_digest(input)
        || receipt.command != ["extract", "--force", "--code-only", "--no-cluster"]
        || receipt.provider_api_calls != 0
        || receipt.api_keys_read != 0
        || receipt.query_logs != 0
        || receipt.watcher
        || receipt.git_hooks
        || receipt.mcp_registration
        || receipt.network_requests != 0
    {
        return Err(WikiError::Verification(
            "Graphify receipt violates the approved code-only boundary".to_owned(),
        ));
    }
    Ok(())
}

fn graphify_dependency_lock_digest() -> Result<String, WikiError> {
    if GRAPHIFY_DEPENDENCY_LOCK.is_empty() {
        return Err(WikiError::InvalidInput(
            "Graphify code sidecar is unsupported on this platform".to_owned(),
        ));
    }
    let value: Value = serde_json::from_slice(GRAPHIFY_DEPENDENCY_LOCK)
        .map_err(|error| WikiError::Verification(format!("invalid Graphify lock: {error}")))?;
    if value["schema_version"] != 1
        || value["package"] != "graphifyy==0.9.47"
        || value["python"] != "3.12"
        || value["files"].as_array().map(Vec::len) != Some(30)
    {
        return Err(WikiError::Verification(
            "Graphify dependency lock contract mismatch".to_owned(),
        ));
    }
    let canonical = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| WikiError::Verification(format!("canonicalize Graphify lock: {error}")))?;
    Ok(sha256_digest(&canonical))
}

fn graphify_consent(scope: &str) -> Result<GraphifyConsent, WikiError> {
    let dependency_lock_digest = graphify_dependency_lock_digest()?;
    let command = vec![
        "extract".to_owned(),
        "--force".to_owned(),
        "--code-only".to_owned(),
        "--no-cluster".to_owned(),
    ];
    let payload = json!({
        "schema_version": 1,
        "scope": scope,
        "package_version": GRAPHIFY_VERSION,
        "wheel_digest": GRAPHIFY_WHEEL_DIGEST,
        "dependency_lock_digest": dependency_lock_digest,
        "command": command,
        "automatic_install": false,
        "provider_api_calls": 0,
        "api_keys_read": 0,
        "query_logs": 0,
        "watcher": false,
        "git_hooks": false,
        "mcp_registration": false,
        "network_requests": 0
    });
    let consent_digest = sha256_digest(
        &serde_json_canonicalizer::to_vec(&payload)
            .map_err(|error| WikiError::Io(format!("canonicalize Graphify consent: {error}")))?,
    );
    Ok(GraphifyConsent {
        schema_version: 1,
        scope: scope.to_owned(),
        package_version: GRAPHIFY_VERSION.to_owned(),
        wheel_digest: GRAPHIFY_WHEEL_DIGEST.to_owned(),
        dependency_lock_digest,
        command,
        consent_digest,
    })
}

fn persist_graphify_consent(
    target: &Path,
    scope: &str,
    consent: &GraphifyConsent,
) -> Result<Vec<String>, WikiError> {
    let relative = Path::new(graphify_consent_relative(scope)?);
    ensure_graphify_consent_path(target, relative, scope)?;
    let destination = target.join(relative);
    let bytes = serde_json_canonicalizer::to_vec(consent)
        .map_err(|error| WikiError::Io(format!("canonicalize Graphify consent: {error}")))?;
    match fs::symlink_metadata(&destination) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(WikiError::Conflict(
                "Graphify consent destination is not a regular file".to_owned(),
            ));
        }
        Ok(_) => {
            let existing = fs::read(&destination)
                .map_err(|error| WikiError::Io(format!("read Graphify consent: {error}")))?;
            if existing == bytes {
                return Ok(Vec::new());
            }
            return Err(WikiError::Conflict(
                "Graphify consent already contains different bytes".to_owned(),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(WikiError::Io(format!("inspect Graphify consent: {error}")));
        }
    }
    let parent = destination
        .parent()
        .ok_or_else(|| WikiError::Io("Graphify consent has no parent".to_owned()))?;
    fs::create_dir_all(parent)
        .map_err(|error| WikiError::Io(format!("create Graphify consent parent: {error}")))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| WikiError::Io(format!("create Graphify consent staging: {error}")))?;
    temporary
        .write_all(&bytes)
        .map_err(|error| WikiError::Io(format!("write Graphify consent staging: {error}")))?;
    temporary
        .persist(&destination)
        .map_err(|error| WikiError::Io(format!("activate Graphify consent: {error}")))?;
    Ok(vec![relative.to_string_lossy().into_owned()])
}

fn load_graphify_consent(target: &Path, scope: &str) -> Result<GraphifyConsent, WikiError> {
    let bytes = read_bytes_bounded(
        &target.join(graphify_consent_relative(scope)?),
        "Graphify consent",
        64 * 1024,
    )?;
    let consent: GraphifyConsent = serde_json::from_slice(&bytes)
        .map_err(|error| WikiError::Verification(format!("invalid Graphify consent: {error}")))?;
    let expected = graphify_consent(scope)?;
    if consent.schema_version != 1
        || consent.scope != expected.scope
        || consent.package_version != expected.package_version
        || consent.wheel_digest != expected.wheel_digest
        || consent.dependency_lock_digest != expected.dependency_lock_digest
        || consent.command != expected.command
        || consent.consent_digest != expected.consent_digest
    {
        return Err(WikiError::Verification(
            "Graphify consent binding mismatch".to_owned(),
        ));
    }
    Ok(consent)
}

fn remove_graphify_consent(target: &Path, scope: &str) -> Result<bool, WikiError> {
    let relative = Path::new(graphify_consent_relative(scope)?);
    ensure_graphify_consent_path(target, relative, scope)?;
    let destination = target.join(relative);
    match fs::symlink_metadata(&destination) {
        Ok(metadata) if metadata.is_file() => {
            fs::remove_file(destination)
                .map_err(|error| WikiError::Io(format!("remove Graphify consent: {error}")))?;
            Ok(true)
        }
        Ok(_) => Err(WikiError::Conflict(
            "Graphify consent destination is not a regular file".to_owned(),
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(WikiError::Io(format!("inspect Graphify consent: {error}"))),
    }
}

fn graphify_consent_relative(scope: &str) -> Result<&'static str, WikiError> {
    match scope {
        "project" => Ok(PROJECT_GRAPHIFY_CONSENT_RELATIVE),
        "source" => Ok(SOURCE_GRAPHIFY_CONSENT_RELATIVE),
        _ => Err(WikiError::InvalidInput(
            "Graphify consent scope is unsupported".to_owned(),
        )),
    }
}

fn ensure_graphify_consent_path(
    target: &Path,
    relative: &Path,
    scope: &str,
) -> Result<(), WikiError> {
    if scope == "source" {
        ensure_graph_owned_path(target, relative, scope).map_err(|error| {
            WikiError::Conflict(format!("Graphify consent path is unsafe: {error}"))
        })
    } else {
        ensure_no_symlink_ancestors(target, relative).map_err(|error| {
            WikiError::Conflict(format!("Graphify consent path is unsafe: {error}"))
        })
    }
}

#[cfg(feature = "notion-preview")]
fn run_notion(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let action = arguments.first().map(String::as_str).ok_or_else(|| {
        WikiError::InvalidInput(
            "knowledge notion requires sync, rebuild, retrieve, or write-through".to_owned(),
        )
    })?;
    match action {
        "sync" => run_notion_sync(&arguments[1..], false),
        "rebuild" => run_notion_sync(&arguments[1..], true),
        "retrieve" => run_notion_retrieve(&arguments[1..]),
        "write-through" => run_notion_write_through(&arguments[1..]),
        _ => Err(WikiError::InvalidInput(format!(
            "unknown knowledge notion action: {action}"
        ))),
    }
}

#[cfg(feature = "notion-preview")]
fn parse_notion_sync_inputs<'a>(
    arguments: &'a [String],
    extra: &[&str],
) -> Result<NotionSyncInputs<'a>, WikiError> {
    let mut allowed = vec!["--user-root", "--capability", "--snapshot"];
    allowed.extend(extra.iter().copied());
    let options = parse_options(arguments, &allowed)?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let capability = read_json_bounded::<NotionCapabilityReceipt>(
        Path::new(required(&options, "--capability")?),
        "Notion capability receipt",
    )?;
    let snapshot = read_json_bounded::<NotionSyncRequest>(
        Path::new(required(&options, "--snapshot")?),
        "complete Notion inventory",
    )?;
    validate_notion_backend(&user_root, &capability)?;
    Ok((user_root, capability, snapshot, options))
}

#[cfg(feature = "notion-preview")]
fn run_notion_sync(arguments: &[String], rebuild: bool) -> Result<KnowledgeResult, WikiError> {
    let (user_root, capability, snapshot, _) = parse_notion_sync_inputs(arguments, &[])?;
    let store = RagStore::open(&user_root)?;
    let outcome = sync_notion_and_publish(&store, &capability, &snapshot, rebuild)
        .map_err(|error| map_notion_error(&error))?;
    let code = if rebuild {
        "hive.notion-rebuilt"
    } else {
        "hive.notion-fresh"
    };
    let message = if rebuild {
        "complete Notion inventory rebuilt the disposable SQLite projection"
    } else {
        "complete Notion inventory freshness preflight completed"
    };
    let digest = outcome.store.manifest_digest.clone();
    Ok(success(
        "SyncNotionKnowledge",
        code,
        message,
        outcome.store.changed_paths.clone(),
        NOTION_LEDGER_RELATIVE,
        &digest,
        notion_sync_data(&outcome),
    ))
}

#[cfg(feature = "notion-preview")]
fn run_notion_retrieve(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let (user_root, capability, snapshot, options) = parse_notion_sync_inputs(
        arguments,
        &[
            "--request",
            "--scope",
            "--query",
            "--top-k",
            "--byte-budget",
        ],
    )?;
    let mut request = parse_retrieval_request(&options, RetrievalScope::Global)?;
    if !matches!(request.scope, RetrievalScope::Global) {
        return Err(WikiError::InvalidInput(
            "Notion mode retrieval currently requires the selected Notion scope (global)"
                .to_owned(),
        ));
    }
    request.current_collection_id = None;
    let store = RagStore::open(&user_root)?;
    let freshness = sync_notion_and_publish(&store, &capability, &snapshot, false)
        .map_err(|error| map_notion_error(&error))?;
    let retrieval =
        retrieve_notion_persisted(&store, &request).map_err(|error| map_notion_error(&error))?;
    let digest = retrieval.manifest_digest.clone();
    Ok(success(
        "RetrieveNotionKnowledge",
        "hive.notion-retrieved-fresh",
        "fresh Notion preflight and bounded SQLite retrieval completed",
        freshness.store.changed_paths.clone(),
        SHARED_INDEX_RELATIVE,
        &digest,
        json!({
            "freshness": notion_sync_data(&freshness),
            "retrieval": retrieval,
        }),
    ))
}

#[cfg(feature = "notion-preview")]
fn run_notion_write_through(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let (user_root, capability, snapshot, options) =
        parse_notion_sync_inputs(arguments, &["--write-receipt"])?;
    let receipt = read_json_bounded::<NotionWriteReceipt>(
        Path::new(required(&options, "--write-receipt")?),
        "Notion write receipt",
    )?;
    validate_write_receipt(&capability, &snapshot, &receipt)
        .map_err(|error| map_notion_error(&error))?;
    let store = RagStore::open(&user_root)?;
    let outcome = sync_notion_and_publish(&store, &capability, &snapshot, false)
        .map_err(|error| map_notion_error(&error))?;
    let digest = outcome.store.manifest_digest.clone();
    Ok(success(
        "WriteThroughNotionKnowledge",
        "hive.notion-write-through-complete",
        "host-confirmed Notion canonical write was validated and written through to SQLite",
        outcome.store.changed_paths.clone(),
        NOTION_LEDGER_RELATIVE,
        &digest,
        json!({
            "write": {
                "operation": receipt.operation,
                "page_id": receipt.page_id,
                "revision": receipt.revision,
            },
            "freshness": notion_sync_data(&outcome),
        }),
    ))
}

#[cfg(feature = "notion-preview")]
fn validate_notion_backend(
    user_root: &Path,
    capability: &NotionCapabilityReceipt,
) -> Result<(), WikiError> {
    let wiki = super::user_setup::operational_wiki_preferences(user_root).map_err(|error| {
        WikiError::Verification(format!("cannot authorize Notion mode: {error}"))
    })?;
    if !wiki.enabled || wiki.backend != super::user_setup::WikiBackend::Notion {
        return Err(WikiError::Conflict(
            "knowledge notion requires enabled global Wiki backend notion".to_owned(),
        ));
    }
    let Some(notion) = wiki.notion else {
        return Err(WikiError::Verification(
            "Notion backend configuration is incomplete".to_owned(),
        ));
    };
    if !notion.local_index_consent {
        return Err(WikiError::Verification(
            "Notion backend lacks consent for local derived SQLite content".to_owned(),
        ));
    }
    if notion.workspace_id != capability.workspace_id || notion.scope_id != capability.scope_id {
        return Err(WikiError::Conflict(
            "Notion capability receipt differs from the selected workspace or scope".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(feature = "notion-preview")]
fn map_notion_error(error: &hive_wiki::notion::NotionError) -> WikiError {
    WikiError::Verification(error.to_string())
}

#[cfg(feature = "notion-preview")]
fn notion_sync_data(outcome: &NotionPersistedOutcome) -> Value {
    json!({
        "adapter": outcome.sync.projection.adapter,
        "workspace_id": outcome.sync.projection.workspace_id,
        "scope_id": outcome.sync.projection.scope_id,
        "generation": outcome.store.generation,
        "changed_page_ids": outcome.sync.changed_page_ids,
        "tombstoned_page_ids": outcome.sync.tombstoned_page_ids,
        "remote_requests": outcome.sync.remote_requests,
        "document_count": outcome.sync.projection.documents.len(),
        "chunk_count": outcome.sync.projection.artifact.chunk_count,
        "store": outcome.store,
    })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ScanPhase {
    Inventory,
    Candidates,
    Apply,
}

struct ScanArguments {
    target: PathBuf,
    include_untracked: bool,
    prior_inventory: Option<PathBuf>,
    review: Option<PathBuf>,
    user_root: Option<PathBuf>,
    phase: ScanPhase,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewedClaimsFile {
    schema_version: u32,
    inventory_digest: String,
    claims: Vec<ReviewedClaim>,
}

#[allow(clippy::too_many_lines)]
fn run_scan(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let arguments = parse_scan_arguments(arguments)?;
    let prior = arguments
        .prior_inventory
        .as_deref()
        .map(|path| read_json_bounded::<ScanInventory>(path, "prior scan inventory"))
        .transpose()?;
    let outcome = scan_directory(
        &arguments.target,
        arguments.include_untracked,
        prior.as_ref(),
    )?;
    let data = match arguments.phase {
        ScanPhase::Inventory => json!({
            "phase": "inventory",
            "scan": outcome,
        }),
        ScanPhase::Candidates | ScanPhase::Apply => {
            let review_path = arguments.review.as_deref().ok_or_else(|| {
                WikiError::InvalidInput(
                    "scan candidate/apply phase requires a review file".to_owned(),
                )
            })?;
            let review = read_json_bounded::<ReviewedClaimsFile>(review_path, "reviewed claims")?;
            if review.schema_version != 1 {
                return Err(WikiError::InvalidInput(
                    "reviewed claims schema_version must be 1".to_owned(),
                ));
            }
            if review.inventory_digest != outcome.inventory.inventory_digest {
                return Err(WikiError::Conflict(
                    "reviewed claims are stale for the current directory inventory".to_owned(),
                ));
            }
            let validated = validate_claims(&outcome.inventory, &review.claims)?;
            // Candidate review is a promise that the exact same review can be applied while the
            // inventory remains current. Keep its storage-level safety checks identical to apply.
            validate_reviewed_claims_for_apply(&validated)?;
            if arguments.phase == ScanPhase::Apply {
                let user_root = arguments.user_root.as_deref().ok_or_else(|| {
                    WikiError::InvalidInput("scan --apply requires --user-root".to_owned())
                })?;
                require_shared_wiki_enabled(user_root)?;
                let store = RagStore::open(user_root)?;
                let canonical_target = hive_wiki::shared::canonical_root(&arguments.target)?;
                let alias = canonical_target
                    .file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or("scanned-directory")
                    .to_owned();
                let committed = store.register_scanned_collection_atomic(
                    CollectionRegistration {
                        collection_id: None,
                        kind: CollectionKind::Directory,
                        state: CollectionState::Attached,
                        aliases: vec![alias],
                        local_locator: Some(canonical_target),
                        source_project_id: None,
                        default_visibility: CollectionVisibility::ProjectPrivate,
                        portable_identity: None,
                        reviewed_inventory_digest: Some(outcome.inventory.inventory_digest.clone()),
                    },
                    &validated,
                )?;
                let automatic_promotion = store.auto_promote_reviewed_scan_claims_atomic(
                    &committed.collection.collection_id,
                )?;
                let changed_paths = committed
                    .store
                    .changed_paths
                    .iter()
                    .chain(automatic_promotion.store.changed_paths.iter())
                    .cloned()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                return Ok(success(
                    "ScanKnowledge",
                    "hive.knowledge-scan-applied",
                    "agent-reviewed directory claims were stored and safe-general claims promoted without target mutation",
                    changed_paths,
                    SHARED_INDEX_RELATIVE,
                    &automatic_promotion.store.manifest_digest,
                    json!({
                        "phase": "apply",
                        "scan": outcome,
                        "validated_claims": validated,
                        "collection": committed.collection,
                        "store": automatic_promotion.store.clone(),
                        "automatic_promotion": automatic_promotion,
                        "target_mutated": false,
                    }),
                ));
            }
            json!({
                "phase": "candidates",
                "scan": outcome,
                "validated_claims": validated,
                "canonical_mutation": false,
            })
        }
    };
    let digest = outcome.inventory.inventory_digest.clone();
    Ok(success(
        "ScanKnowledge",
        "hive.knowledge-scan-complete",
        "directory scan completed without target mutation",
        Vec::new(),
        ".hive/knowledge/Collections",
        &digest,
        data,
    ))
}

fn run_export(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &["--user-root", "--scope", "--bundle", "--replace-backup"],
    )?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let bundle = PathBuf::from(required(&options, "--bundle")?);
    let scope = parse_bundle_scope(required(&options, "--scope")?)?;
    let publish_mode =
        optional(&options, "--replace-backup").map_or(BundlePublishMode::CreateOnly, |name| {
            BundlePublishMode::Replace {
                backup_file_name: name.into(),
            }
        });
    let exported = export_bundle(
        &user_root,
        scope,
        &bundle,
        &publish_mode,
        BundleLimits::default(),
    )?;
    let changed_paths = if exported.disposition == BundleExportDisposition::Unchanged {
        Vec::new()
    } else {
        vec![bundle.display().to_string()]
    };
    let digest = exported.archive_sha256.clone();
    Ok(success(
        "ExportKnowledge",
        "hive.knowledge-exported",
        "portable canonical knowledge bundle published",
        changed_paths,
        &bundle.display().to_string(),
        &digest,
        serde_json::to_value(exported).map_err(|error| WikiError::Io(error.to_string()))?,
    ))
}

fn run_import(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let dry_run = arguments
        .iter()
        .filter(|argument| *argument == "--dry-run")
        .count();
    let apply = arguments
        .iter()
        .filter(|argument| *argument == "--apply")
        .count();
    if dry_run + apply != 1 {
        return Err(WikiError::InvalidInput(
            "knowledge import requires exactly one of --dry-run or --apply".to_owned(),
        ));
    }
    let filtered = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--dry-run" && argument.as_str() != "--apply")
        .cloned()
        .collect::<Vec<_>>();
    let options = parse_options(&filtered, &["--user-root", "--bundle"])?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let bundle = PathBuf::from(required(&options, "--bundle")?);
    run_import_mode(
        &user_root,
        &bundle,
        if apply == 1 {
            BundleImportMode::Apply
        } else {
            BundleImportMode::DryRun
        },
    )
}

fn run_import_mode(
    user_root: &Path,
    bundle: &Path,
    mode: BundleImportMode,
) -> Result<KnowledgeResult, WikiError> {
    let imported = import_bundle(user_root, bundle, mode, BundleLimits::default())?;
    import_report(bundle, imported)
}

fn import_report(
    bundle: &Path,
    imported: BundleImportResult,
) -> Result<KnowledgeResult, WikiError> {
    let digest = imported.manifest_sha256.clone();
    Ok(success(
        "ImportKnowledge",
        "hive.knowledge-imported",
        "portable canonical knowledge bundle validated and merged",
        imported.changed_paths.clone(),
        &bundle.display().to_string(),
        &digest,
        serde_json::to_value(imported).map_err(|error| WikiError::Io(error.to_string()))?,
    ))
}

fn parse_bundle_scope(value: &str) -> Result<BundleScope, WikiError> {
    match value {
        "global" => Ok(BundleScope::Global),
        "shared" => Ok(BundleScope::Shared),
        "all-portable" => Ok(BundleScope::AllPortable),
        _ => value
            .strip_prefix("project:")
            .filter(|id| !id.is_empty())
            .map(|id| BundleScope::Project { id: id.to_owned() })
            .or_else(|| {
                value
                    .strip_prefix("collection:")
                    .filter(|id| !id.is_empty())
                    .map(|id| BundleScope::Collection { id: id.to_owned() })
            })
            .ok_or_else(|| WikiError::InvalidInput(format!("unsupported bundle scope: {value}"))),
    }
}

fn parse_scan_arguments(arguments: &[String]) -> Result<ScanArguments, WikiError> {
    let mut target = None;
    let mut include_untracked = false;
    let mut prior_inventory = None;
    let mut review = None;
    let mut user_root = None;
    let mut phase = None;
    let mut output = None;
    let mut index = 0_usize;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        index += 1;
        match option {
            "--include-untracked" if !include_untracked => include_untracked = true,
            "--inventory" if phase.is_none() => phase = Some(ScanPhase::Inventory),
            "--candidates" | "--apply" if phase.is_none() => {
                let value = arguments.get(index).ok_or_else(|| {
                    WikiError::InvalidInput(format!("missing value for {option}"))
                })?;
                review = Some(PathBuf::from(value));
                phase = Some(if option == "--apply" {
                    ScanPhase::Apply
                } else {
                    ScanPhase::Candidates
                });
                index += 1;
            }
            "--target" | "--prior-inventory" | "--user-root" | "--output" => {
                let value = arguments.get(index).ok_or_else(|| {
                    WikiError::InvalidInput(format!("missing value for {option}"))
                })?;
                let slot = match option {
                    "--target" => &mut target,
                    "--prior-inventory" => &mut prior_inventory,
                    "--user-root" => &mut user_root,
                    "--output" => &mut output,
                    _ => unreachable!(),
                };
                if slot.replace(PathBuf::from(value)).is_some() {
                    return Err(WikiError::InvalidInput(format!(
                        "duplicate scan option: {option}"
                    )));
                }
                index += 1;
            }
            "--include-untracked" | "--inventory" | "--candidates" | "--apply" => {
                return Err(WikiError::InvalidInput(format!(
                    "duplicate or conflicting scan option: {option}"
                )));
            }
            _ => {
                return Err(WikiError::InvalidInput(format!(
                    "unknown scan option: {option}"
                )));
            }
        }
    }
    if output.as_deref().and_then(|path| path.to_str()) != Some("json") {
        return Err(WikiError::InvalidInput(
            "knowledge scan requires --output json".to_owned(),
        ));
    }
    Ok(ScanArguments {
        target: target.ok_or_else(|| {
            WikiError::InvalidInput("knowledge scan requires --target".to_owned())
        })?,
        include_untracked,
        prior_inventory,
        review,
        user_root,
        phase: phase.ok_or_else(|| {
            WikiError::InvalidInput(
                "knowledge scan requires exactly one of --inventory, --candidates, or --apply"
                    .to_owned(),
            )
        })?,
    })
}

fn read_json_bounded<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &str,
) -> Result<T, WikiError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| WikiError::Io(format!("cannot inspect {label}: {error}")))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 4 * 1024 * 1024
    {
        return Err(WikiError::InvalidInput(format!(
            "{label} must be a regular file no larger than 4 MiB"
        )));
    }
    let bytes =
        fs::read(path).map_err(|error| WikiError::Io(format!("cannot read {label}: {error}")))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| WikiError::InvalidInput(format!("invalid {label}: {error}")))
}

fn ensure_rag_registry(user_root: &Path, store: &RagStore) -> Result<StoreCommit, WikiError> {
    let project_registry = ensure_project_registry(user_root)?;
    let projects = load_project_registry(user_root)?;
    let mut commit = store.sync_project_registry(&projects)?;
    commit.changed_paths.extend(project_registry.changed_paths);
    commit.changed_paths.sort();
    commit.changed_paths.dedup();
    Ok(commit)
}

pub(crate) fn run_index(arguments: &[String]) -> ExitCode {
    if is_help(arguments) {
        print!("{KNOWLEDGE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = if arguments.first().map(String::as_str) == Some("rebuild") {
        parse_options(&arguments[1..], &["--target", "--user-root"]).and_then(|options| {
            let user_root = optional(&options, "--user-root");
            let target = optional(&options, "--target");
            let (changed_paths, logical_digest, data) = match (target, user_root) {
                (None, Some(user_root)) => {
                    require_shared_wiki_enabled(Path::new(user_root))?;
                    let store = RagStore::open(Path::new(user_root))?;
                    let initialized = ensure_rag_registry(Path::new(user_root), &store)?;
                    let mut outcome = store.rebuild()?;
                    outcome.changed_paths.extend(initialized.changed_paths);
                    outcome.changed_paths.sort();
                    outcome.changed_paths.dedup();
                    let changed_paths = outcome.changed_paths.clone();
                    let logical_digest = outcome.manifest_digest.clone();
                    let data = serde_json::to_value(&outcome)
                        .map_err(|error| WikiError::Io(error.to_string()))?;
                    (changed_paths, logical_digest, data)
                }
                (Some(target), None) => {
                    authorize_legacy_target(Path::new(target))?;
                    let outcome = rebuild_index(&PathBuf::from(target))?;
                    let changed_paths = outcome.changed_paths.clone();
                    let logical_digest = outcome.logical_digest.clone();
                    let data = serde_json::to_value(&outcome)
                        .map_err(|error| WikiError::Io(error.to_string()))?;
                    (changed_paths, logical_digest, data)
                }
                _ => {
                    return Err(WikiError::InvalidInput(
                        "index rebuild requires exactly one of --target or --user-root".to_owned(),
                    ));
                }
            };
            Ok(success(
                "RebuildKnowledgeIndex",
                "hive.index-rebuilt",
                "knowledge index rebuilt from canonical sources",
                changed_paths,
                SHARED_INDEX_RELATIVE,
                &logical_digest,
                data,
            ))
        })
    } else {
        Err(WikiError::InvalidInput(
            "index requires the rebuild action".to_owned(),
        ))
    }
    .unwrap_or_else(|error| failure("RebuildKnowledgeIndex", &error));
    emit(&result);
    ExitCode::from(result.exit_code)
}

fn run_remember(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--user-root",
            "--request",
            "--user-statement",
            "--claim-key",
            "--kind",
        ],
    )?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let request = match (
        optional(&options, "--request"),
        optional(&options, "--user-statement"),
    ) {
        (Some(request), None) => {
            for option in ["--claim-key", "--kind"] {
                if optional(&options, option).is_some() {
                    return Err(WikiError::InvalidInput(format!(
                        "{option} requires --user-statement, not --request"
                    )));
                }
            }
            read_json_bounded::<RememberRequest>(Path::new(request), "remember request")?
        }
        (None, Some(statement)) => user_statement_remember_request(&options, statement)?,
        (Some(_), Some(_)) => {
            return Err(WikiError::InvalidInput(
                "remember accepts exactly one of --request or --user-statement".to_owned(),
            ));
        }
        (None, None) => {
            return Err(WikiError::InvalidInput(
                "remember requires --request or --user-statement".to_owned(),
            ));
        }
    };
    require_shared_wiki_enabled(&user_root)?;

    // Independent inserts can finish shape and secret validation before initialization.
    // Explicit supersedes need the authenticated current claim set for validation.
    if request.supersedes.is_empty() {
        plan_remember(&[], &request, 1).map_err(map_rag_error)?;
    }
    let store = RagStore::open(&user_root)?;
    let initialized = ensure_rag_registry(&user_root, &store)?;
    let revision = initialized
        .generation
        .checked_add(1)
        .ok_or_else(|| WikiError::Conflict("RAG generation is exhausted".to_owned()))?;
    let snapshot = store.load_canonical_snapshot(revision)?;
    let plan = plan_remember(&snapshot.claims, &request, revision).map_err(map_rag_error)?;
    let committed = store.apply_remember_plan(&plan)?;
    let mut changed_paths = initialized.changed_paths;
    changed_paths.extend(committed.changed_paths.clone());
    Ok(success(
        "RememberKnowledge",
        "hive.knowledge-remembered",
        "agent-reviewed durable knowledge was written to canonical Markdown",
        changed_paths,
        SHARED_INDEX_RELATIVE,
        &committed.manifest_digest,
        json!({"plan": plan, "store": committed}),
    ))
}

fn user_statement_remember_request(
    options: &[(&str, &str)],
    normalized_fact: &str,
) -> Result<RememberRequest, WikiError> {
    let claim_key = required(options, "--claim-key")?;
    let kind = match optional(options, "--kind").unwrap_or("preference") {
        "project-profile" => ClaimKind::ProjectProfile,
        "decision" => ClaimKind::Decision,
        "convention" => ClaimKind::Convention,
        "preference" => ClaimKind::Preference,
        "workflow" => ClaimKind::Workflow,
        value => {
            return Err(WikiError::InvalidInput(format!(
                "--kind must be project-profile, decision, convention, preference, or workflow; got {value}"
            )));
        }
    };
    let source = format!("request:{claim_key}");
    Ok(RememberRequest {
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        claim_key: claim_key.to_owned(),
        claim_id: None,
        locator: format!("user-root/{claim_key}"),
        kind,
        status: AssertionStatus::UserStated,
        visibility: RagVisibility::Shared,
        normalized_fact: normalized_fact.to_owned(),
        provenance: ClaimProvenance {
            source_kind: RememberSourceKind::UserStatement,
            summary: "Bounded automatic user-statement capture".to_owned(),
            locator: source.clone(),
            digest: sha256_digest(normalized_fact.as_bytes()),
        },
        sources: vec![source],
        supersedes: Vec::new(),
        expected_active_digest: None,
        observed_at: None,
        verified_at: None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ConfidentialAuthorizationRecord {
    schema_version: u32,
    authorization_id: String,
    token_digest: String,
    canonical_action_digest: String,
    authorization_binding_digest: String,
    action: AuthorizationAction,
    issued_at_unix_seconds: u64,
    expires_at_unix_seconds: u64,
    nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum AuthorizationAction {
    ConfidentialRetrieval {
        collection_id: String,
        current_collection_id: String,
        request_binding_digest: String,
    },
    ConfidentialVectorBuild {
        collection_id: String,
        current_collection_id: String,
        request_binding_digest: String,
    },
    CollectionMapping {
        operation: String,
        collection_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        canonical_target_locator: Option<String>,
        registry_digest: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct AuthorizationTombstone {
    schema_version: u32,
    authorization_id: String,
    token_digest: String,
    record_digest: String,
    canonical_action_digest: String,
    nonce: String,
    consumed_at_unix_seconds: u64,
    expires_at_unix_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct AuthorizationReservation {
    schema_version: u32,
    authorization_id: String,
    canonical_action_digest: String,
    nonce: String,
}

#[derive(Serialize)]
struct RetrievalActionBinding<'a> {
    schema_version: u32,
    action: &'static str,
    normalized_query: &'a str,
    query_expansions: &'a [String],
    scope: &'a RetrievalScope,
    resolved_scope_collection_id: &'a str,
    current_target_digest: &'a str,
    current_collection_id: &'a str,
    collection_id: &'a str,
    top_k: usize,
    byte_budget: usize,
}

#[derive(Serialize)]
struct CollectionMappingBinding<'a> {
    schema_version: u32,
    action: &'static str,
    operation: &'a str,
    collection_id: &'a str,
    canonical_target_locator: Option<&'a str>,
    registry_digest: &'a str,
}

#[derive(Serialize)]
struct AuthorizationBinding<'a> {
    schema_version: u32,
    canonical_action_digest: &'a str,
    capability_snapshot_digest: Option<&'a str>,
    usage_snapshot_digest: Option<&'a str>,
    expires_at_unix_seconds: u64,
    nonce: &'a str,
}

struct AuthorizationConsumption {
    collection_id: String,
    changed_path: String,
}

#[allow(clippy::too_many_lines)]
fn run_authorize_confidential(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let confirmation_count = arguments
        .iter()
        .filter(|argument| argument.as_str() == "--confirm-current-action")
        .count();
    if confirmation_count != 1 {
        return Err(WikiError::InvalidInput(
            "confidential authorization requires one --confirm-current-action".to_owned(),
        ));
    }
    let filtered = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--confirm-current-action")
        .cloned()
        .collect::<Vec<_>>();
    let options = parse_options(
        &filtered,
        &[
            "--user-root",
            "--target",
            "--collection",
            "--request",
            "--scope",
            "--query",
            "--top-k",
            "--byte-budget",
            "--capabilities",
            "--usage",
            "--expires-at",
            "--nonce",
        ],
    )?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let target = PathBuf::from(required(&options, "--target")?);
    let nonce = required(&options, "--nonce")?;
    validate_authorization_nonce(nonce)?;
    let expires_at = required(&options, "--expires-at")?
        .parse::<u64>()
        .map_err(|_| WikiError::InvalidInput("--expires-at must be Unix seconds".to_owned()))?;
    let now = unix_now()?;
    if expires_at <= now || expires_at.saturating_sub(now) > MAX_AUTHORIZATION_TTL_SECONDS {
        return Err(WikiError::InvalidInput(format!(
            "confidential authorization expiry must be within {MAX_AUTHORIZATION_TTL_SECONDS} seconds"
        )));
    }
    require_shared_wiki_enabled(&user_root)?;
    let store = RagStore::open(&user_root)?;
    let registry = store.load_registry()?;
    let (canonical_target, current_collection_id) =
        derive_current_collection_authority(&registry, &target)?;
    let collection_id = resolve_collection_reference(
        &registry,
        required(&options, "--collection")?,
        "confidential collection",
    )?;
    let default_scope = if collection_id == current_collection_id {
        RetrievalScope::Auto
    } else {
        RetrievalScope::Collection(collection_id.clone())
    };
    let mut request = parse_retrieval_request(&options, default_scope)?;
    request.current_collection_id = Some(current_collection_id.clone());
    let resolved_scope_collection_id =
        resolve_retrieval_collection(&registry, &request.scope, &current_collection_id)?;
    if resolved_scope_collection_id != collection_id {
        return Err(WikiError::InvalidInput(
            "confidential authorization collection must match the exact effective retrieval scope"
                .to_owned(),
        ));
    }
    let capability_snapshot_digest = read_snapshot_digest(
        Path::new(required(&options, "--capabilities")?),
        "capability snapshot",
    )?;
    let usage_snapshot_digest =
        read_snapshot_digest(Path::new(required(&options, "--usage")?), "usage snapshot")?;
    let current_target_digest = canonical_target_digest(&canonical_target);
    let request_binding_digest = canonical_digest(
        &RetrievalActionBinding {
            schema_version: 1,
            action: "confidential-retrieval",
            normalized_query: &request.query,
            query_expansions: &request.query_expansions,
            scope: &request.scope,
            resolved_scope_collection_id: &resolved_scope_collection_id,
            current_target_digest: &current_target_digest,
            current_collection_id: &current_collection_id,
            collection_id: &collection_id,
            top_k: request.top_k,
            byte_budget: request.byte_budget,
        },
        "confidential retrieval action",
    )?;
    let authorization_binding_digest = canonical_digest(
        &AuthorizationBinding {
            schema_version: 1,
            canonical_action_digest: &request_binding_digest,
            capability_snapshot_digest: Some(&capability_snapshot_digest),
            usage_snapshot_digest: Some(&usage_snapshot_digest),
            expires_at_unix_seconds: expires_at,
            nonce,
        },
        "confidential authorization binding",
    )?;
    let authorization_id = generate_authorization_secret()?;
    let token = generate_authorization_secret()?;
    let record = ConfidentialAuthorizationRecord {
        schema_version: 1,
        authorization_id: authorization_id.clone(),
        token_digest: sha256_digest(token.as_bytes()),
        canonical_action_digest: request_binding_digest.clone(),
        authorization_binding_digest: authorization_binding_digest.clone(),
        action: AuthorizationAction::ConfidentialRetrieval {
            collection_id: collection_id.clone(),
            current_collection_id: current_collection_id.clone(),
            request_binding_digest: request_binding_digest.clone(),
        },
        issued_at_unix_seconds: now,
        expires_at_unix_seconds: expires_at,
        nonce: nonce.to_owned(),
    };
    let (relative, record_digest) = issue_authorization(&user_root, &store, &record)?;
    Ok(success(
        "AuthorizeConfidentialKnowledge",
        "hive.knowledge-confidential-authorized",
        "one-time confidential knowledge authorization issued",
        vec![relative.display().to_string()],
        &relative.display().to_string(),
        &record_digest,
        json!({
            "authorization_id": authorization_id,
            "authorization_token": token,
            "collection_id": collection_id,
            "current_collection_id": current_collection_id,
            "request_binding_digest": request_binding_digest,
            "authorization_binding_digest": authorization_binding_digest,
            "usage_snapshot_digest": usage_snapshot_digest,
            "capability_snapshot_digest": capability_snapshot_digest,
            "expires_at_unix_seconds": record.expires_at_unix_seconds,
            "nonce": record.nonce,
            "one_time": true,
        }),
    ))
}

#[allow(clippy::too_many_lines)]
fn run_authorize_collection(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let confirmation_count = arguments
        .iter()
        .filter(|argument| argument.as_str() == "--confirm-current-action")
        .count();
    if confirmation_count != 1 {
        return Err(WikiError::InvalidInput(
            "collection mapping authorization requires one --confirm-current-action".to_owned(),
        ));
    }
    let filtered = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--confirm-current-action")
        .cloned()
        .collect::<Vec<_>>();
    let options = parse_options(
        &filtered,
        &[
            "--user-root",
            "--operation",
            "--collection",
            "--target",
            "--expires-at",
            "--nonce",
        ],
    )?;
    let operation = required(&options, "--operation")?;
    if !matches!(operation, "attach" | "map" | "detach") {
        return Err(WikiError::InvalidInput(
            "collection mapping operation must be attach, map, or detach".to_owned(),
        ));
    }
    let target = optional(&options, "--target").map(PathBuf::from);
    validate_collection_target_shape(operation, target.as_deref())?;
    let nonce = required(&options, "--nonce")?;
    validate_authorization_nonce(nonce)?;
    let expires_at = parse_authorization_expiry(required(&options, "--expires-at")?)?;
    let now = unix_now()?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    require_shared_wiki_enabled(&user_root)?;
    let store = RagStore::open(&user_root)?;
    let (registry, registry_digest) = store.load_registry_snapshot()?;
    let collection_id =
        resolve_collection_reference(&registry, required(&options, "--collection")?, "collection")?;
    let canonical_target = canonical_collection_target(target.as_deref())?;
    let canonical_target_locator = canonical_target.as_deref().map(canonical_path_locator);
    let canonical_action_digest = canonical_digest(
        &CollectionMappingBinding {
            schema_version: 1,
            action: "collection-mapping",
            operation,
            collection_id: &collection_id,
            canonical_target_locator: canonical_target_locator.as_deref(),
            registry_digest: &registry_digest,
        },
        "collection mapping action",
    )?;
    let authorization_binding_digest = canonical_digest(
        &AuthorizationBinding {
            schema_version: 1,
            canonical_action_digest: &canonical_action_digest,
            capability_snapshot_digest: None,
            usage_snapshot_digest: None,
            expires_at_unix_seconds: expires_at,
            nonce,
        },
        "collection mapping authorization binding",
    )?;
    let authorization_id = generate_authorization_secret()?;
    let token = generate_authorization_secret()?;
    let record = ConfidentialAuthorizationRecord {
        schema_version: 1,
        authorization_id: authorization_id.clone(),
        token_digest: sha256_digest(token.as_bytes()),
        canonical_action_digest: canonical_action_digest.clone(),
        authorization_binding_digest: authorization_binding_digest.clone(),
        action: AuthorizationAction::CollectionMapping {
            operation: operation.to_owned(),
            collection_id: collection_id.clone(),
            canonical_target_locator: canonical_target_locator.clone(),
            registry_digest: registry_digest.clone(),
        },
        issued_at_unix_seconds: now,
        expires_at_unix_seconds: expires_at,
        nonce: nonce.to_owned(),
    };
    let (relative, record_digest) = issue_authorization(&user_root, &store, &record)?;
    Ok(success(
        "MapKnowledgeCollection",
        "hive.knowledge-collection-mapping-authorized",
        "one-time collection mapping authorization issued",
        vec![relative.display().to_string()],
        &relative.display().to_string(),
        &record_digest,
        json!({
            "authorization_id": authorization_id,
            "authorization_token": token,
            "operation": operation,
            "collection_id": collection_id,
            "canonical_target_locator": canonical_target_locator,
            "registry_digest": registry_digest,
            "canonical_action_digest": canonical_action_digest,
            "authorization_binding_digest": authorization_binding_digest,
            "expires_at_unix_seconds": expires_at,
            "nonce": nonce,
            "one_time": true,
        }),
    ))
}

fn run_collection(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let action = arguments.first().map(String::as_str).ok_or_else(|| {
        WikiError::InvalidInput("knowledge collection requires attach, map, or detach".to_owned())
    })?;
    if !matches!(action, "attach" | "map" | "detach") {
        return Err(WikiError::InvalidInput(format!(
            "unknown knowledge collection action: {action}"
        )));
    }
    let options = parse_options(
        &arguments[1..],
        &[
            "--user-root",
            "--collection",
            "--target",
            "--authorization-id",
            "--authorization-token",
        ],
    )?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let target = optional(&options, "--target").map(PathBuf::from);
    validate_collection_target_shape(action, target.as_deref())?;
    require_shared_wiki_enabled(&user_root)?;
    let store = RagStore::open(&user_root)?;
    let (registry, registry_digest) = store.load_registry_snapshot()?;
    let collection_id =
        resolve_collection_reference(&registry, required(&options, "--collection")?, "collection")?;
    let canonical_target = canonical_collection_target(target.as_deref())?;
    let consumption = verify_and_consume_collection_authorization(
        &user_root,
        &store,
        action,
        &collection_id,
        canonical_target.as_deref(),
        &registry_digest,
        required(&options, "--authorization-id")?,
        required(&options, "--authorization-token")?,
    )?;
    let committed = store.set_collection_attachment(
        &collection_id,
        canonical_target.as_deref(),
        &registry_digest,
    )?;
    let digest = committed.store.manifest_digest.clone();
    let code = if action == "detach" {
        "hive.knowledge-collection-detached"
    } else {
        "hive.knowledge-collection-attached"
    };
    let message = if action == "detach" {
        "knowledge collection detached without changing its stable identity"
    } else {
        "knowledge collection mapped to a verified local target"
    };
    let mut changed_paths = committed.store.changed_paths.clone();
    changed_paths.push(consumption.changed_path);
    Ok(success(
        "MapKnowledgeCollection",
        code,
        message,
        changed_paths,
        hive_wiki::store::COLLECTION_REGISTRY_RELATIVE,
        &digest,
        serde_json::to_value(committed).map_err(|error| WikiError::Io(error.to_string()))?,
    ))
}

#[allow(clippy::too_many_lines)]
fn run_retrieve(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--user-root",
            "--target",
            "--request",
            "--scope",
            "--query",
            "--top-k",
            "--byte-budget",
            "--mode",
            "--authorization-id",
            "--authorization-token",
            "--capabilities",
            "--usage",
        ],
    )?;
    let mode = optional(&options, "--mode").unwrap_or("fts");
    if !["fts", "semantic"].contains(&mode) {
        return Err(WikiError::InvalidInput(
            "--mode must be fts or semantic".to_owned(),
        ));
    }
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let target = PathBuf::from(required(&options, "--target")?);
    let mut request = parse_retrieval_request(&options, RetrievalScope::Auto)?;
    require_shared_wiki_enabled(&user_root)?;
    let store = RagStore::open(&user_root)?;
    let registry = store.load_registry()?;
    let (canonical_target, current_collection_id) =
        derive_optional_current_collection_authority(&registry, &target)?;
    request
        .current_collection_id
        .clone_from(&current_collection_id);
    let authorization_fields = [
        "--authorization-id",
        "--authorization-token",
        "--capabilities",
        "--usage",
    ];
    let authorization_count = authorization_fields
        .iter()
        .filter(|option| optional(&options, option).is_some())
        .count();
    if authorization_count != 0 && authorization_count != authorization_fields.len() {
        return Err(WikiError::InvalidInput(
            "confidential retrieval requires authorization ID, token, capability snapshot, and usage snapshot together"
                .to_owned(),
        ));
    }
    let mut changed_paths = Vec::new();
    if authorization_count == authorization_fields.len() {
        let current_collection_id = current_collection_id.as_deref().ok_or_else(|| {
            WikiError::InvalidInput(
                "confidential retrieval requires an attached current project collection".to_owned(),
            )
        })?;
        let consumption = verify_and_consume_authorization(
            &user_root,
            &store,
            &registry,
            &request,
            &canonical_target,
            current_collection_id,
            required(&options, "--authorization-id")?,
            required(&options, "--authorization-token")?,
            Path::new(required(&options, "--capabilities")?),
            Path::new(required(&options, "--usage")?),
        )?;
        request.confidential_collection_id = Some(consumption.collection_id);
        changed_paths.push(consumption.changed_path);
    }
    let result = if mode == "semantic" {
        vector::retrieve(&user_root, &store, &request)?
    } else {
        serde_json::to_value(store.retrieve(&request)?)
            .map_err(|error| WikiError::Io(error.to_string()))?
    };
    let digest = result["manifest_digest"]
        .as_str()
        .ok_or_else(|| WikiError::Verification("retrieval manifest is absent".to_owned()))?
        .to_owned();
    Ok(success(
        "RetrieveKnowledge",
        "hive.knowledge-retrieved",
        "bounded knowledge retrieval completed",
        changed_paths,
        SHARED_INDEX_RELATIVE,
        &digest,
        result,
    ))
}

fn parse_retrieval_request(
    options: &[(&str, &str)],
    default_scope: RetrievalScope,
) -> Result<RetrievalRequest, WikiError> {
    let request_path = optional(options, "--request");
    let inline_present = ["--scope", "--query", "--top-k", "--byte-budget"]
        .iter()
        .any(|option| optional(options, option).is_some());
    if request_path.is_some() && inline_present {
        return Err(WikiError::InvalidInput(
            "--request cannot be combined with inline retrieval options".to_owned(),
        ));
    }
    let mut request = if let Some(path) = request_path {
        read_json_bounded::<RetrievalRequest>(Path::new(path), "retrieval request")?
    } else {
        RetrievalRequest {
            scope: optional(options, "--scope").map_or(Ok(default_scope), |value| {
                RetrievalScope::from_str(value).map_err(map_rag_error)
            })?,
            current_collection_id: None,
            query: required(options, "--query")?.to_owned(),
            query_expansions: Vec::new(),
            top_k: parse_bounded_usize(optional(options, "--top-k"), 5, 1, 100, "--top-k")?,
            byte_budget: parse_bounded_usize(
                optional(options, "--byte-budget"),
                16 * 1024,
                1,
                1024 * 1024,
                "--byte-budget",
            )?,
            confidential_collection_id: None,
        }
    };
    if request.current_collection_id.is_some() || request.confidential_collection_id.is_some() {
        return Err(WikiError::InvalidInput(
            "retrieval request files cannot self-assert current or confidential collection authority"
                .to_owned(),
        ));
    }
    request.query = normalize_retrieval_text(&request.query, "query")?;
    let mut seen = BTreeSet::new();
    let mut expansions = Vec::new();
    for expansion in &request.query_expansions {
        let normalized = normalize_retrieval_text(expansion, "query expansion")?;
        if seen.insert(normalized.clone()) {
            expansions.push(normalized);
        }
    }
    if expansions.len() > 8 {
        return Err(WikiError::InvalidInput(
            "query_expansions exceeds 8 normalized entries".to_owned(),
        ));
    }
    if request.top_k == 0 || request.top_k > 100 {
        return Err(WikiError::InvalidInput(
            "top_k must be between 1 and 100".to_owned(),
        ));
    }
    if request.byte_budget == 0 || request.byte_budget > 1024 * 1024 {
        return Err(WikiError::InvalidInput(
            "byte_budget must be between 1 and 1048576".to_owned(),
        ));
    }
    request.query_expansions = expansions;
    Ok(request)
}

fn normalize_retrieval_text(value: &str, label: &str) -> Result<String, WikiError> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() || normalized.len() > 4096 {
        return Err(WikiError::InvalidInput(format!(
            "{label} must contain 1 through 4096 UTF-8 bytes"
        )));
    }
    Ok(normalized)
}

fn resolve_retrieval_collection(
    registry: &hive_wiki::collection::CollectionRegistry,
    scope: &RetrievalScope,
    current_collection_id: &str,
) -> Result<String, WikiError> {
    match scope {
        RetrievalScope::Auto => Ok(current_collection_id.to_owned()),
        RetrievalScope::Project(project_id) => match registry.resolve_project(project_id) {
            CollectionResolution::Resolved(collection_id) => Ok(collection_id),
            CollectionResolution::Unknown => {
                Err(WikiError::InvalidInput("unknown project scope".to_owned()))
            }
            CollectionResolution::Ambiguous(ids) => Err(WikiError::Conflict(format!(
                "ambiguous project scope resolves to {}",
                ids.join(", ")
            ))),
        },
        RetrievalScope::Collection(reference) => {
            resolve_collection_reference(registry, reference, "collection scope")
        }
        RetrievalScope::Global | RetrievalScope::AllVisible => {
            Ok(USER_ROOT_COLLECTION_ID.to_owned())
        }
    }
}

fn canonical_digest(value: &impl Serialize, label: &str) -> Result<String, WikiError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|error| WikiError::Io(format!("cannot canonicalize {label}: {error}")))?;
    Ok(sha256_digest(&bytes))
}

fn canonical_path_locator(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn canonical_target_digest(path: &Path) -> String {
    sha256_digest(canonical_path_locator(path).as_bytes())
}

fn parse_authorization_expiry(value: &str) -> Result<u64, WikiError> {
    let expires_at = value
        .parse::<u64>()
        .map_err(|_| WikiError::InvalidInput("--expires-at must be Unix seconds".to_owned()))?;
    let now = unix_now()?;
    if expires_at <= now || expires_at.saturating_sub(now) > MAX_AUTHORIZATION_TTL_SECONDS {
        return Err(WikiError::InvalidInput(format!(
            "authorization expiry must be within {MAX_AUTHORIZATION_TTL_SECONDS} seconds"
        )));
    }
    Ok(expires_at)
}

fn validate_collection_target_shape(
    operation: &str,
    target: Option<&Path>,
) -> Result<(), WikiError> {
    if matches!(operation, "attach" | "map") && target.is_none() {
        return Err(WikiError::InvalidInput(
            "knowledge collection attach/map requires --target".to_owned(),
        ));
    }
    if operation == "detach" && target.is_some() {
        return Err(WikiError::InvalidInput(
            "knowledge collection detach does not accept --target".to_owned(),
        ));
    }
    Ok(())
}

fn canonical_collection_target(target: Option<&Path>) -> Result<Option<PathBuf>, WikiError> {
    target
        .map(|target| {
            ensure_consumer_target(target)
                .map_err(|error| WikiError::Conflict(error.to_string()))?;
            hive_wiki::shared::canonical_root(target)
        })
        .transpose()
}

fn derive_current_collection_authority(
    registry: &hive_wiki::collection::CollectionRegistry,
    target: &Path,
) -> Result<(PathBuf, String), WikiError> {
    let (canonical_target, collection_id) =
        derive_optional_current_collection_authority(registry, target)?;
    collection_id.map_or_else(
        || {
            Err(WikiError::InvalidInput(
                "current target is not attached to exactly one knowledge collection".to_owned(),
            ))
        },
        |collection_id| Ok((canonical_target, collection_id)),
    )
}

fn derive_optional_current_collection_authority(
    registry: &hive_wiki::collection::CollectionRegistry,
    target: &Path,
) -> Result<(PathBuf, Option<String>), WikiError> {
    ensure_consumer_target(target).map_err(|error| WikiError::Conflict(error.to_string()))?;
    let canonical_target = hive_wiki::shared::canonical_root(target)?;
    let mut matches = registry
        .collections
        .iter()
        .filter(|collection| collection.state == CollectionState::Attached)
        .filter_map(|collection| {
            let locator = collection.local_locator.as_deref()?;
            hive_wiki::shared::canonical_root(Path::new(locator))
                .is_ok_and(|root| root == canonical_target)
                .then_some(collection.collection_id.clone())
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    match matches.as_slice() {
        [collection_id] => Ok((canonical_target, Some(collection_id.clone()))),
        [] => Ok((canonical_target, None)),
        _ => Err(WikiError::Conflict(
            "current target maps to multiple knowledge collections".to_owned(),
        )),
    }
}

fn resolve_collection_reference(
    registry: &hive_wiki::collection::CollectionRegistry,
    reference: &str,
    label: &str,
) -> Result<String, WikiError> {
    match registry.resolve_collection(reference) {
        CollectionResolution::Resolved(collection_id) => Ok(collection_id),
        CollectionResolution::Unknown => Err(WikiError::InvalidInput(format!(
            "unknown {label} `{reference}`"
        ))),
        CollectionResolution::Ambiguous(ids) => Err(WikiError::Conflict(format!(
            "ambiguous {label} `{reference}` resolves to {}",
            ids.join(", ")
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
fn verify_and_consume_authorization(
    user_root: &Path,
    store: &RagStore,
    registry: &hive_wiki::collection::CollectionRegistry,
    request: &RetrievalRequest,
    canonical_target: &Path,
    current_collection_id: &str,
    authorization_id: &str,
    token: &str,
    capability_snapshot: &Path,
    usage_snapshot: &Path,
) -> Result<AuthorizationConsumption, WikiError> {
    validate_authorization_credentials(authorization_id, token)?;
    let (canonical_user_root, root) = authorization_root(user_root, false)?;
    let _lock = store.acquire_authorization_lock()?;
    let relative = authorization_record_relative(authorization_id, false)?;
    reject_consumed_authorization(&root, authorization_id)?;
    let bytes = read_authorization_record(&canonical_user_root, &root, &relative)?;
    let record = parse_authorization_record(&bytes)?;
    validate_authorization_record_common(&record, authorization_id, token)?;
    let AuthorizationAction::ConfidentialRetrieval {
        collection_id,
        current_collection_id: record_current_collection_id,
        request_binding_digest,
    } = &record.action
    else {
        return Err(WikiError::Verification(
            "authorization is bound to another action kind".to_owned(),
        ));
    };
    let resolved_scope_collection_id =
        resolve_retrieval_collection(registry, &request.scope, current_collection_id)?;
    if resolved_scope_collection_id != *collection_id
        || record_current_collection_id != current_collection_id
    {
        return Err(WikiError::Verification(
            "confidential authorization is bound to another collection scope".to_owned(),
        ));
    }
    if !registry
        .collections
        .iter()
        .any(|collection| collection.collection_id == *collection_id)
    {
        return Err(WikiError::Verification(
            "confidential authorization collection is no longer registered".to_owned(),
        ));
    }
    let capability_snapshot_digest =
        read_snapshot_digest(capability_snapshot, "capability snapshot")?;
    let usage_snapshot_digest = read_snapshot_digest(usage_snapshot, "usage snapshot")?;
    let current_target_digest = canonical_target_digest(canonical_target);
    let expected_action_digest = canonical_digest(
        &RetrievalActionBinding {
            schema_version: 1,
            action: "confidential-retrieval",
            normalized_query: &request.query,
            query_expansions: &request.query_expansions,
            scope: &request.scope,
            resolved_scope_collection_id: &resolved_scope_collection_id,
            current_target_digest: &current_target_digest,
            current_collection_id,
            collection_id,
            top_k: request.top_k,
            byte_budget: request.byte_budget,
        },
        "confidential retrieval action",
    )?;
    let expected_binding_digest = canonical_digest(
        &AuthorizationBinding {
            schema_version: 1,
            canonical_action_digest: &expected_action_digest,
            capability_snapshot_digest: Some(&capability_snapshot_digest),
            usage_snapshot_digest: Some(&usage_snapshot_digest),
            expires_at_unix_seconds: record.expires_at_unix_seconds,
            nonce: &record.nonce,
        },
        "confidential authorization binding",
    )?;
    if expected_action_digest != record.canonical_action_digest
        || request_binding_digest != &expected_action_digest
        || expected_binding_digest != record.authorization_binding_digest
    {
        return Err(WikiError::Verification(
            "confidential authorization is bound to another normalized retrieval action".to_owned(),
        ));
    }
    let consumed =
        consume_authorization_record(&canonical_user_root, &root, &relative, &bytes, &record)?;
    Ok(AuthorizationConsumption {
        collection_id: collection_id.clone(),
        changed_path: consumed.display().to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
fn verify_and_consume_collection_authorization(
    user_root: &Path,
    store: &RagStore,
    operation: &str,
    collection_id: &str,
    canonical_target: Option<&Path>,
    registry_digest: &str,
    authorization_id: &str,
    token: &str,
) -> Result<AuthorizationConsumption, WikiError> {
    validate_authorization_credentials(authorization_id, token)?;
    let (canonical_user_root, root) = authorization_root(user_root, false)?;
    let _lock = store.acquire_authorization_lock()?;
    let relative = authorization_record_relative(authorization_id, false)?;
    reject_consumed_authorization(&root, authorization_id)?;
    let bytes = read_authorization_record(&canonical_user_root, &root, &relative)?;
    let record = parse_authorization_record(&bytes)?;
    validate_authorization_record_common(&record, authorization_id, token)?;
    let AuthorizationAction::CollectionMapping {
        operation: expected_operation,
        collection_id: expected_collection_id,
        canonical_target_locator,
        registry_digest: expected_registry_digest,
    } = &record.action
    else {
        return Err(WikiError::Verification(
            "authorization is bound to another action kind".to_owned(),
        ));
    };
    let target_locator = canonical_target.map(canonical_path_locator);
    let expected_action_digest = canonical_digest(
        &CollectionMappingBinding {
            schema_version: 1,
            action: "collection-mapping",
            operation,
            collection_id,
            canonical_target_locator: target_locator.as_deref(),
            registry_digest,
        },
        "collection mapping action",
    )?;
    let expected_binding_digest = canonical_digest(
        &AuthorizationBinding {
            schema_version: 1,
            canonical_action_digest: &expected_action_digest,
            capability_snapshot_digest: None,
            usage_snapshot_digest: None,
            expires_at_unix_seconds: record.expires_at_unix_seconds,
            nonce: &record.nonce,
        },
        "collection mapping authorization binding",
    )?;
    if expected_operation != operation
        || expected_collection_id != collection_id
        || canonical_target_locator != &target_locator
        || expected_registry_digest != registry_digest
        || record.canonical_action_digest != expected_action_digest
        || record.authorization_binding_digest != expected_binding_digest
    {
        return Err(WikiError::Verification(
            "collection mapping authorization is stale or bound to another exact action".to_owned(),
        ));
    }
    let consumed =
        consume_authorization_record(&canonical_user_root, &root, &relative, &bytes, &record)?;
    Ok(AuthorizationConsumption {
        collection_id: collection_id.to_owned(),
        changed_path: consumed.display().to_string(),
    })
}

fn validate_authorization_credentials(
    authorization_id: &str,
    token: &str,
) -> Result<(), WikiError> {
    if !valid_authorization_secret(authorization_id) || !valid_authorization_secret(token) {
        return Err(WikiError::Verification(
            "authorization token is malformed or forged".to_owned(),
        ));
    }
    Ok(())
}

fn parse_authorization_record(bytes: &[u8]) -> Result<ConfidentialAuthorizationRecord, WikiError> {
    serde_json::from_slice(bytes).map_err(|_| {
        WikiError::Verification("authorization record is malformed or forged".to_owned())
    })
}

fn validate_authorization_record_common(
    record: &ConfidentialAuthorizationRecord,
    authorization_id: &str,
    token: &str,
) -> Result<(), WikiError> {
    let now = unix_now()?;
    if record.schema_version != 1
        || record.authorization_id != authorization_id
        || record.token_digest != sha256_digest(token.as_bytes())
        || !valid_sha256_digest(&record.canonical_action_digest)
        || !valid_sha256_digest(&record.authorization_binding_digest)
        || record.issued_at_unix_seconds > now
        || now >= record.expires_at_unix_seconds
        || record.expires_at_unix_seconds <= record.issued_at_unix_seconds
        || record
            .expires_at_unix_seconds
            .saturating_sub(record.issued_at_unix_seconds)
            > MAX_AUTHORIZATION_TTL_SECONDS
        || validate_authorization_nonce(&record.nonce).is_err()
    {
        return Err(WikiError::Verification(
            "authorization is stale, forged, or outside its one-action lifetime".to_owned(),
        ));
    }
    Ok(())
}

fn reject_consumed_authorization(root: &Dir, authorization_id: &str) -> Result<(), WikiError> {
    let consumed = authorization_record_relative(authorization_id, true)?;
    match root.symlink_metadata(&consumed) {
        Ok(_) => Err(WikiError::Verification(
            "authorization was already consumed".to_owned(),
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(WikiError::Verification(format!(
            "cannot inspect authorization tombstone: {error}"
        ))),
    }
}

fn consume_authorization_record(
    canonical_user_root: &Path,
    root: &Dir,
    active_relative: &Path,
    active_bytes: &[u8],
    record: &ConfidentialAuthorizationRecord,
) -> Result<PathBuf, WikiError> {
    let consumed_relative = authorization_record_relative(&record.authorization_id, true)?;
    let tombstone = AuthorizationTombstone {
        schema_version: 1,
        authorization_id: record.authorization_id.clone(),
        token_digest: record.token_digest.clone(),
        record_digest: sha256_digest(active_bytes),
        canonical_action_digest: record.canonical_action_digest.clone(),
        nonce: record.nonce.clone(),
        consumed_at_unix_seconds: unix_now()?,
        expires_at_unix_seconds: record.expires_at_unix_seconds,
    };
    let bytes = serialize_authorization_value(&tombstone)?;
    write_authorization_record(root, &consumed_relative, &bytes).map_err(|error| {
        WikiError::Verification(format!(
            "authorization was already consumed or changed: {error}"
        ))
    })?;
    ensure_no_symlink_ancestors(canonical_user_root, &consumed_relative)
        .map_err(|error| WikiError::Conflict(error.to_string()))?;
    let archived_relative = authorization_consumed_record_relative(&record.authorization_id)?;
    root.rename(active_relative, root, &archived_relative)
        .map_err(|error| {
            WikiError::Verification(format!(
                "cannot archive consumed authorization record: {error}"
            ))
        })?;
    Ok(consumed_relative)
}

fn authorization_root(user_root: &Path, create: bool) -> Result<(PathBuf, Dir), WikiError> {
    let canonical = hive_wiki::shared::canonical_root(user_root)?;
    ensure_no_symlink_ancestors(&canonical, Path::new(KNOWLEDGE_AUTHORIZATION_RELATIVE))
        .map_err(|error| WikiError::Conflict(error.to_string()))?;
    let root = Dir::open_ambient_dir(&canonical, ambient_authority())
        .map_err(|error| WikiError::Io(format!("cannot pin knowledge user root: {error}")))?;
    if create {
        for relative in [
            KNOWLEDGE_AUTHORIZATION_CONSUMED_RELATIVE,
            KNOWLEDGE_AUTHORIZATION_BINDINGS_RELATIVE,
        ] {
            root.create_dir_all(relative).map_err(|error| {
                WikiError::Io(format!(
                    "cannot create knowledge authorization directory: {error}"
                ))
            })?;
        }
    } else {
        for relative in [
            KNOWLEDGE_AUTHORIZATION_RELATIVE,
            KNOWLEDGE_AUTHORIZATION_CONSUMED_RELATIVE,
            KNOWLEDGE_AUTHORIZATION_BINDINGS_RELATIVE,
        ] {
            let metadata = root.metadata(relative).map_err(|error| {
                WikiError::Verification(format!(
                    "knowledge authorization runtime is missing or inaccessible: {error}"
                ))
            })?;
            if !metadata.is_dir() {
                return Err(WikiError::Verification(
                    "knowledge authorization runtime is not a directory".to_owned(),
                ));
            }
        }
    }
    for relative in [
        KNOWLEDGE_AUTHORIZATION_CONSUMED_RELATIVE,
        KNOWLEDGE_AUTHORIZATION_BINDINGS_RELATIVE,
    ] {
        ensure_no_symlink_ancestors(&canonical, Path::new(relative))
            .map_err(|error| WikiError::Conflict(error.to_string()))?;
    }
    Ok((canonical, root))
}

fn reject_authorization_record_exhaustion(user_root: &Path) -> Result<(), WikiError> {
    let directory = user_root.join(KNOWLEDGE_AUTHORIZATION_RELATIVE);
    let mut count = 0_usize;
    for entry in fs::read_dir(&directory).map_err(|error| {
        WikiError::Io(format!(
            "cannot enumerate knowledge authorizations: {error}"
        ))
    })? {
        let entry = entry.map_err(|error| {
            WikiError::Io(format!("cannot enumerate knowledge authorization: {error}"))
        })?;
        let metadata = entry.file_type().map_err(|error| {
            WikiError::Io(format!("cannot inspect knowledge authorization: {error}"))
        })?;
        if metadata.is_symlink() {
            return Err(WikiError::Conflict(
                "knowledge authorization directory contains a symlink".to_owned(),
            ));
        }
        if metadata.is_file() {
            count += 1;
        }
    }
    if count >= MAX_AUTHORIZATION_RECORDS {
        return Err(WikiError::Conflict(format!(
            "active knowledge authorizations exceed {MAX_AUTHORIZATION_RECORDS} records"
        )));
    }
    Ok(())
}

fn generate_authorization_secret() -> Result<String, WikiError> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut random = [0_u8; 32];
    getrandom::fill(&mut random).map_err(|error| {
        WikiError::Io(format!("cannot obtain OS authorization entropy: {error}"))
    })?;
    let mut encoded = String::with_capacity(64);
    for byte in random {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(encoded)
}

fn serialize_authorization_record(
    record: &ConfidentialAuthorizationRecord,
) -> Result<Vec<u8>, WikiError> {
    serialize_authorization_value(record)
}

fn serialize_authorization_value(value: &impl Serialize) -> Result<Vec<u8>, WikiError> {
    let mut bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|error| WikiError::Io(format!("cannot serialize authorization: {error}")))?;
    bytes.push(b'\n');
    if bytes.len() > MAX_AUTHORIZATION_BYTES {
        return Err(WikiError::InvalidInput(
            "confidential authorization record exceeds its byte limit".to_owned(),
        ));
    }
    Ok(bytes)
}

fn issue_authorization(
    user_root: &Path,
    store: &RagStore,
    record: &ConfidentialAuthorizationRecord,
) -> Result<(PathBuf, String), WikiError> {
    let _lock = store.acquire_authorization_lock()?;
    let (canonical_user_root, root) = authorization_root(user_root, true)?;
    reject_authorization_record_exhaustion(&canonical_user_root)?;
    let record_bytes = serialize_authorization_record(record)?;
    let relative = authorization_record_relative(&record.authorization_id, false)?;
    let reservation = AuthorizationReservation {
        schema_version: 1,
        authorization_id: record.authorization_id.clone(),
        canonical_action_digest: record.canonical_action_digest.clone(),
        nonce: record.nonce.clone(),
    };
    let reservation_relative = authorization_reservation_relative(record)?;
    let reservation_bytes = serialize_authorization_value(&reservation)?;
    write_authorization_record(&root, &reservation_relative, &reservation_bytes).map_err(
        |error| {
            WikiError::Conflict(format!(
                "authorization nonce and exact action were already issued: {error}"
            ))
        },
    )?;
    write_authorization_record(&root, &relative, &record_bytes)?;
    Ok((relative, sha256_digest(&record_bytes)))
}

fn authorization_reservation_relative(
    record: &ConfidentialAuthorizationRecord,
) -> Result<PathBuf, WikiError> {
    let digest = canonical_digest(
        &json!({
            "schema_version": 1,
            "nonce": record.nonce,
            "canonical_action_digest": record.canonical_action_digest,
        }),
        "authorization nonce reservation",
    )?;
    let key = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| WikiError::Io("authorization reservation digest is malformed".to_owned()))?;
    Ok(PathBuf::from(format!(
        "{KNOWLEDGE_AUTHORIZATION_BINDINGS_RELATIVE}/{key}.json"
    )))
}

fn authorization_record_relative(id: &str, consumed: bool) -> Result<PathBuf, WikiError> {
    if !valid_authorization_secret(id) {
        return Err(WikiError::Verification(
            "confidential authorization ID is malformed".to_owned(),
        ));
    }
    Ok(PathBuf::from(if consumed {
        format!("{KNOWLEDGE_AUTHORIZATION_CONSUMED_RELATIVE}/{id}.json")
    } else {
        format!("{KNOWLEDGE_AUTHORIZATION_RELATIVE}/{id}.json")
    }))
}

fn authorization_consumed_record_relative(id: &str) -> Result<PathBuf, WikiError> {
    if !valid_authorization_secret(id) {
        return Err(WikiError::Verification(
            "confidential authorization ID is malformed".to_owned(),
        ));
    }
    Ok(PathBuf::from(format!(
        "{KNOWLEDGE_AUTHORIZATION_CONSUMED_RELATIVE}/{id}.record.json"
    )))
}

fn write_authorization_record(root: &Dir, relative: &Path, bytes: &[u8]) -> Result<(), WikiError> {
    let mut options = CapOpenOptions::new();
    options.write(true).create_new(true);
    options.follow(FollowSymlinks::No);
    let mut file = root.open_with(relative, &options).map_err(|error| {
        WikiError::Conflict(format!(
            "cannot create knowledge authorization record: {error}"
        ))
    })?;
    file.write_all(bytes).map_err(|error| {
        WikiError::Io(format!(
            "cannot write knowledge authorization record: {error}"
        ))
    })?;
    file.sync_all().map_err(|error| {
        WikiError::Io(format!(
            "cannot sync knowledge authorization record: {error}"
        ))
    })?;
    protect_authorization_file(&file)?;
    file.sync_all().map_err(|error| {
        WikiError::Io(format!(
            "cannot sync protected knowledge authorization record: {error}"
        ))
    })
}

fn read_authorization_record(
    canonical_user_root: &Path,
    root: &Dir,
    relative: &Path,
) -> Result<Vec<u8>, WikiError> {
    let mut options = CapOpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    let file = root.open_with(relative, &options).map_err(|error| {
        WikiError::Verification(format!(
            "confidential authorization is missing, replayed, or inaccessible: {error}"
        ))
    })?;
    let metadata = file.metadata().map_err(|error| {
        WikiError::Io(format!(
            "cannot inspect confidential authorization: {error}"
        ))
    })?;
    if !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_AUTHORIZATION_BYTES as u64
        || !metadata.permissions().readonly()
    {
        return Err(WikiError::Verification(
            "authorization record is not a bounded protected regular file".to_owned(),
        ));
    }
    let std_file = file.into_std();
    let std_metadata = std_file.metadata().map_err(|error| {
        WikiError::Io(format!("cannot inspect authorization file handle: {error}"))
    })?;
    verify_authorization_file_identity(canonical_user_root, &metadata, &std_metadata)?;
    let mut bytes = Vec::new();
    std_file
        .take((MAX_AUTHORIZATION_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            WikiError::Io(format!("cannot read confidential authorization: {error}"))
        })?;
    if bytes.len() > MAX_AUTHORIZATION_BYTES {
        return Err(WikiError::Verification(
            "confidential authorization record exceeds its byte limit".to_owned(),
        ));
    }
    Ok(bytes)
}

fn protect_authorization_file(file: &cap_std::fs::File) -> Result<(), WikiError> {
    let metadata = file.metadata().map_err(|error| {
        WikiError::Io(format!("cannot inspect authorization permissions: {error}"))
    })?;
    let mut permissions = metadata.permissions();
    #[cfg(unix)]
    {
        use cap_std::fs::PermissionsExt;
        permissions.set_mode(0o400);
    }
    #[cfg(windows)]
    permissions.set_readonly(true);
    file.set_permissions(permissions)
        .map_err(|error| WikiError::Io(format!("cannot protect authorization record: {error}")))
}

fn verify_authorization_file_identity(
    canonical_user_root: &Path,
    capability_metadata: &cap_std::fs::Metadata,
    std_metadata: &fs::Metadata,
) -> Result<(), WikiError> {
    if CapMetadataExt::nlink(capability_metadata) != 1 {
        return Err(WikiError::Verification(
            "authorization record has an unsafe hard-link count".to_owned(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let owner = fs::metadata(canonical_user_root.join(KNOWLEDGE_AUTHORIZATION_RELATIVE))
            .map_err(|error| {
                WikiError::Io(format!("cannot inspect authorization owner: {error}"))
            })?;
        if std_metadata.uid() != owner.uid() {
            return Err(WikiError::Verification(
                "authorization record has an unsafe owner or hard-link count".to_owned(),
            ));
        }
    }
    #[cfg(windows)]
    let _ = (canonical_user_root, std_metadata);
    Ok(())
}

fn read_snapshot_digest(path: &Path, label: &str) -> Result<String, WikiError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| WikiError::Io(format!("cannot inspect {label}: {error}")))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_AUTHORIZATION_SNAPSHOT_BYTES
    {
        return Err(WikiError::InvalidInput(format!(
            "{label} must be a regular JSON file no larger than 1 MiB"
        )));
    }
    let bytes =
        fs::read(path).map_err(|error| WikiError::Io(format!("cannot read {label}: {error}")))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| WikiError::InvalidInput(format!("invalid {label}: {error}")))?;
    if !value
        .as_object()
        .and_then(|object| object.get("schema_version"))
        .is_some_and(Value::is_number)
    {
        return Err(WikiError::InvalidInput(format!(
            "{label} requires a numeric schema_version"
        )));
    }
    Ok(sha256_digest(&bytes))
}

fn validate_authorization_nonce(nonce: &str) -> Result<(), WikiError> {
    if !(16..=128).contains(&nonce.len())
        || !nonce
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(WikiError::InvalidInput(
            "authorization nonce must be 16..=128 ASCII letters, digits, hyphens, or underscores"
                .to_owned(),
        ));
    }
    Ok(())
}

fn valid_authorization_secret(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_sha256_digest(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(valid_authorization_secret)
}

fn unix_now() -> Result<u64, WikiError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| WikiError::Verification("system clock predates Unix epoch".to_owned()))
}

fn parse_bounded_usize(
    value: Option<&str>,
    default: usize,
    minimum: usize,
    maximum: usize,
    label: &str,
) -> Result<usize, WikiError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = value
        .parse::<usize>()
        .map_err(|_| WikiError::InvalidInput(format!("{label} must be an integer")))?;
    if !(minimum..=maximum).contains(&parsed) {
        return Err(WikiError::InvalidInput(format!(
            "{label} must be in {minimum}..={maximum}"
        )));
    }
    Ok(parsed)
}

fn map_rag_error(error: RagError) -> WikiError {
    match error {
        RagError::InvalidInput(message) => WikiError::InvalidInput(message),
        RagError::Conflict(message) => WikiError::Conflict(message),
        RagError::RepairRequired(message) => WikiError::Verification(message),
        RagError::Io(message) => WikiError::Io(message),
        RagError::Sqlite(message) => WikiError::Sqlite(message),
    }
}

fn run_ingest(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &["--target", "--source", "--wiki", "--user-root"],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let source = PathBuf::from(required(&options, "--source")?);
    let wiki = PathBuf::from(required(&options, "--wiki")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_mutation_target(&target, Path::new(root), true))
        .transpose()?;
    let _shared_operation_lock = shared
        .as_ref()
        .map(acquire_shared_operation_lock)
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let prepared = shared
        .as_ref()
        .map(prepare_shared_mutation)
        .transpose()?
        .unwrap_or_default();
    let outcome = if let Some(shared) = &shared {
        ingest_shared(
            &target,
            &source,
            &wiki,
            &shared.user_root,
            &shared.namespace,
        )?
    } else {
        ingest(&target, &source, &wiki)?
    };
    let mutation =
        serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = finish_shared_mutation(
        &target,
        shared.as_ref(),
        outcome.changed_paths.into_iter().chain(prepared).collect(),
        mutation,
        ".hive/knowledge",
    )?;
    Ok(success(
        "IngestKnowledge",
        "hive.knowledge-ingested",
        "Raw revision and Wiki page integrated serially",
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

fn run_add(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let quick_count = arguments
        .iter()
        .filter(|argument| *argument == "--quick")
        .count();
    if quick_count > 1 {
        return Err(WikiError::InvalidInput(
            "duplicate knowledge option: --quick".to_owned(),
        ));
    }
    let filtered = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--quick")
        .cloned()
        .collect::<Vec<_>>();
    let mut result = run_ingest(&filtered)?;
    result.action = "AddKnowledge";
    result.code = "hive.knowledge-added";
    result.message = if quick_count == 1 {
        "agent-reviewed quick Wiki draft added through the canonical ingest path".to_owned()
    } else {
        "Wiki source added through the canonical ingest path".to_owned()
    };
    if let Some(data) = result.data.as_mut().and_then(Value::as_object_mut) {
        data.insert("quick".to_owned(), Value::Bool(quick_count == 1));
    }
    Ok(result)
}

fn run_query(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--text",
            "--tag",
            "--category",
            "--limit",
            "--user-root",
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let text = optional(&options, "--text");
    let tag = optional(&options, "--tag");
    let category = optional(&options, "--category");
    let limit = optional(&options, "--limit").map_or(Ok(20_usize), |value| {
        value
            .parse::<usize>()
            .map_err(|_| WikiError::InvalidInput("query limit must be an integer".to_owned()))
    })?;
    if let Some(user_root) = optional(&options, "--user-root") {
        return run_shared_query(
            &target,
            &PathBuf::from(user_root),
            text,
            tag,
            category,
            limit,
        );
    }
    authorize_legacy_target(&target)?;
    let project_hits = query_filtered(&target, text, tag, category, limit)?;
    let mut seen = BTreeSet::new();
    let mut hits = Vec::new();
    for hit in &project_hits {
        seen.insert(hit.content_digest.clone());
        hits.push(scoped_hit("project", "project-local", hit)?);
    }
    let root_hit_count = 0_usize;
    let data = json!({
        "hits": hits,
        "project_hit_count": project_hits.len(),
        "root_hit_count": root_hit_count,
        "precedence": "project-first"
    });
    let digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    Ok(success(
        "QueryKnowledge",
        "hive.knowledge-query-complete",
        "knowledge query completed",
        Vec::new(),
        ".hive/index/hive.sqlite3",
        &digest,
        data,
    ))
}

fn run_shared_query(
    target: &Path,
    user_root: &Path,
    text: Option<&str>,
    tag: Option<&str>,
    category: Option<&str>,
    limit: usize,
) -> Result<KnowledgeResult, WikiError> {
    ensure_consumer_target(target).map_err(|error| WikiError::Conflict(error.to_string()))?;
    require_shared_wiki_enabled(user_root)?;
    let canonical_user = hive_wiki::shared::canonical_root(user_root)?;
    let canonical_target = hive_wiki::shared::canonical_root(target)?;
    let registry = load_project_registry(user_root)?;
    let current_project_id = if canonical_target == canonical_user {
        None
    } else {
        Some(
            registry
                .projects
                .iter()
                .find(|project| {
                    project.enabled
                        && hive_wiki::shared::canonical_root(&project.root)
                            .is_ok_and(|registered| registered == canonical_target)
                })
                .map(|project| project.id.clone())
                .ok_or_else(|| {
                    WikiError::InvalidInput(
                        "query target is not enabled in the project registry".to_owned(),
                    )
                })?,
        )
    };
    let hits = query_shared_filtered(
        user_root,
        (canonical_target != canonical_user).then_some(canonical_target.as_path()),
        text,
        tag,
        category,
        limit,
    )?;
    let project_hit_count = hits
        .iter()
        .filter(|hit| {
            current_project_id
                .as_ref()
                .is_some_and(|project_id| hit.source_project == *project_id)
        })
        .count();
    let root_hit_count = hits
        .iter()
        .filter(|hit| hit.source_project == "user-root")
        .count();
    let cross_project_hit_count = hits
        .len()
        .saturating_sub(project_hit_count + root_hit_count);
    let data = json!({
        "hits": hits,
        "project_hit_count": project_hit_count,
        "root_hit_count": root_hit_count,
        "cross_project_hit_count": cross_project_hit_count,
        "precedence": "own-project,user-root,shared"
    });
    let digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    Ok(success(
        "QueryKnowledge",
        "hive.knowledge-query-complete",
        "shared knowledge query completed",
        Vec::new(),
        SHARED_INDEX_RELATIVE,
        &digest,
        data,
    ))
}

fn run_list(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &["--target", "--tag", "--category", "--limit", "--user-root"],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let limit = parse_bounded_usize(optional(&options, "--limit"), 20, 1, 100, "--limit")?;
    let pages = if let Some(user_root) = optional(&options, "--user-root") {
        let shared = shared_mutation_target(&target, Path::new(user_root), true)?;
        let current_project =
            (shared.target_kind == SharedTargetKind::RegisteredProject).then_some(target.as_path());
        let tag = optional(&options, "--tag");
        let category = optional(&options, "--category");
        let mut hits = if tag.is_some() || category.is_some() {
            query_shared_filtered(
                &shared.user_root,
                current_project,
                None,
                tag,
                category,
                limit,
            )?
        } else {
            let mut hits = Vec::new();
            for category in [
                "source",
                "entity",
                "concept",
                "comparison",
                "synthesis",
                "question",
                "decision",
                "workflow",
            ] {
                hits.extend(query_shared_filtered(
                    &shared.user_root,
                    current_project,
                    None,
                    None,
                    Some(category),
                    100,
                )?);
            }
            hits
        };
        hits.sort_by(|left, right| {
            (&left.page.id, &left.source_project).cmp(&(&right.page.id, &right.source_project))
        });
        hits.dedup_by(|left, right| {
            left.source_project == right.source_project && left.page.id == right.page.id
        });
        hits.truncate(limit);
        hits.into_iter().map(|hit| hit.page).collect::<Vec<_>>()
    } else {
        authorize_legacy_target(&target)?;
        list_pages(
            &target,
            optional(&options, "--tag"),
            optional(&options, "--category"),
            limit,
        )?
    };
    let data = json!({"pages": pages});
    let digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    Ok(success(
        "ListKnowledge",
        "hive.knowledge-listed",
        "canonical Wiki pages listed in deterministic order",
        Vec::new(),
        SHARED_INDEX_RELATIVE,
        &digest,
        data,
    ))
}

fn run_read(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(arguments, &["--target", "--page-id", "--user-root"])?;
    let target = PathBuf::from(required(&options, "--target")?);
    if let Some(user_root) = optional(&options, "--user-root") {
        shared_mutation_target(&target, Path::new(user_root), true)?;
    } else {
        authorize_legacy_target(&target)?;
    }
    let page = read_page(&target, required(&options, "--page-id")?)?;
    let digest = page.content_digest.clone();
    let locator = page.path.clone();
    let data = serde_json::to_value(page).map_err(|error| WikiError::Io(error.to_string()))?;
    Ok(success(
        "ReadKnowledge",
        "hive.knowledge-read",
        "canonical Wiki page and reciprocal-link evidence read",
        Vec::new(),
        &locator,
        &digest,
        data,
    ))
}

fn run_refresh(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(arguments, &["--target", "--user-root"])?;
    match (
        optional(&options, "--target"),
        optional(&options, "--user-root"),
    ) {
        (None, Some(user_root)) => {
            let user_root = Path::new(user_root);
            require_shared_wiki_enabled(user_root)?;
            let store = RagStore::open(user_root)?;
            let initialized = ensure_rag_registry(user_root, &store)?;
            let mut refreshed = store.rebuild()?;
            refreshed.changed_paths.extend(initialized.changed_paths);
            let digest = refreshed.manifest_digest.clone();
            Ok(success(
                "RefreshKnowledge",
                "hive.knowledge-refreshed",
                "RAG index refreshed from canonical Markdown",
                refreshed.changed_paths.clone(),
                SHARED_INDEX_RELATIVE,
                &digest,
                serde_json::to_value(refreshed)
                    .map_err(|error| WikiError::Io(error.to_string()))?,
            ))
        }
        (Some(target), None) => {
            let target = Path::new(target);
            authorize_legacy_target(target)?;
            let refreshed = rebuild_index(target)?;
            let digest = refreshed.logical_digest.clone();
            Ok(success(
                "RefreshKnowledge",
                "hive.knowledge-refreshed",
                "legacy Wiki index refreshed from canonical Markdown",
                refreshed.changed_paths.clone(),
                ".hive/index/hive.sqlite3",
                &digest,
                serde_json::to_value(refreshed)
                    .map_err(|error| WikiError::Io(error.to_string()))?,
            ))
        }
        _ => Err(WikiError::InvalidInput(
            "knowledge refresh requires exactly one of --target or --user-root".to_owned(),
        )),
    }
}

fn scoped_hit(
    scope: &str,
    provenance: &str,
    hit: &hive_wiki::QueryHit,
) -> Result<Value, WikiError> {
    let mut value = serde_json::to_value(hit).map_err(|error| WikiError::Io(error.to_string()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| WikiError::Io("query hit did not serialize as an object".to_owned()))?;
    object.insert("scope".to_owned(), Value::String(scope.to_owned()));
    object.insert(
        "provenance".to_owned(),
        Value::String(provenance.to_owned()),
    );
    Ok(value)
}

fn run_promote(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    if arguments.iter().any(|argument| argument == "--collection") {
        return run_scan_claim_promote(arguments);
    }
    run_page_promote(arguments)
}

#[allow(clippy::too_many_lines)]
fn run_scan_claim_promote(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let (options, mode, confirmed) = parse_scan_promotion_options(arguments)?;
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    require_shared_wiki_enabled(&user_root)?;
    let store = RagStore::open(&user_root)?;
    let registry = store.load_registry()?;
    let collection_id = resolve_collection_reference(
        &registry,
        required(&options, "--collection")?,
        "promotion collection",
    )?;
    let review_id = optional(&options, "--review-id");
    let candidates = store.preview_reviewed_scan_promotions(&collection_id)?;
    let selected = candidates
        .iter()
        .filter(|claim| {
            review_id.is_none_or(|expected| {
                claim
                    .scan_metadata
                    .as_ref()
                    .is_some_and(|metadata| metadata.review_id == expected)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(WikiError::InvalidInput(
            "no pending reviewed scan promotion matches the request".to_owned(),
        ));
    }
    let snapshot = store.load_canonical_snapshot(1)?;
    let preview_revision = snapshot
        .generation
        .checked_add(1)
        .ok_or_else(|| WikiError::Conflict("RAG generation is exhausted".to_owned()))?;
    if mode == PromotionMode::DryRun {
        let previews = selected
            .iter()
            .map(|source| scan_promotion_preview(source, &snapshot.claims, preview_revision))
            .collect::<Result<Vec<_>, _>>()?;
        let bytes =
            serde_json::to_vec(&previews).map_err(|error| WikiError::Io(error.to_string()))?;
        let digest = sha256_digest(&bytes);
        return Ok(success(
            "PromoteKnowledge",
            "hive.knowledge-scan-promotion-planned",
            "reviewed scan promotion preview completed without canonical mutation",
            Vec::new(),
            ".hive/knowledge/Claims",
            &digest,
            json!({
                "mode": "dry-run",
                "collection_id": collection_id,
                "candidates": previews,
                "canonical_mutation": false,
                "approval_required": true,
            }),
        ));
    }
    if !confirmed {
        return Err(WikiError::InvalidInput(
            "reviewed scan promotion apply requires --confirm-global-promotion".to_owned(),
        ));
    }
    let review_id = review_id.ok_or_else(|| {
        WikiError::InvalidInput("reviewed scan promotion apply requires --review-id".to_owned())
    })?;
    if selected.len() != 1 {
        return Err(WikiError::Conflict(
            "reviewed scan promotion apply must resolve exactly one candidate".to_owned(),
        ));
    }
    let source = &selected[0];
    let expected_source_digest = required(&options, "--expected-source-digest")?;
    if source.digest != expected_source_digest {
        return Err(WikiError::Conflict(
            "reviewed scan claim changed after promotion preview".to_owned(),
        ));
    }
    let request = build_scan_promotion_request(source)?;
    plan_remember(&snapshot.claims, &request, preview_revision).map_err(map_rag_error)?;
    let committed = store.promote_reviewed_scan_claim_atomic(
        &collection_id,
        review_id,
        expected_source_digest,
        &request,
    )?;
    let digest = committed.store.manifest_digest.clone();
    Ok(success(
        "PromoteKnowledge",
        "hive.knowledge-scan-promoted",
        "exact reviewed scan claim promoted after digest-bound approval",
        committed.store.changed_paths.clone(),
        SHARED_INDEX_RELATIVE,
        &digest,
        json!({
            "mode": "apply",
            "collection_id": collection_id,
            "review_id": review_id,
            "expected_source_digest": expected_source_digest,
            "approval": "explicit-global-promotion",
            "redaction": "none-exact-reviewed-fact-preserved",
            "deduplication": "canonical-plan",
            "contradiction": "blocked-unless-explicitly-resolved",
            "replacement": "none",
            "commit": committed,
        }),
    ))
}

fn scan_promotion_preview(
    source: &CanonicalClaim,
    claims: &[CanonicalClaim],
    revision: u64,
) -> Result<Value, WikiError> {
    let request = build_scan_promotion_request(source)?;
    let active_same_key = claims
        .iter()
        .filter(|claim| {
            claim.collection_id == USER_ROOT_COLLECTION_ID
                && claim.status != hive_wiki::rag::AssertionStatus::Superseded
                && claim.claim_key == request.claim_key
        })
        .collect::<Vec<_>>();
    let duplicates = active_same_key
        .iter()
        .filter(|claim| claim.normalized_fact == source.normalized_fact)
        .map(|claim| claim.claim_id.clone())
        .collect::<Vec<_>>();
    let contradictions = active_same_key
        .iter()
        .filter(|claim| claim.normalized_fact != source.normalized_fact)
        .map(|claim| claim.claim_id.clone())
        .collect::<Vec<_>>();
    let plan = plan_remember(claims, &request, revision);
    let (decision, plan_value, blocked_reason) = match plan {
        Ok(plan) => (
            serde_json::to_value(plan.disposition)
                .map_err(|error| WikiError::Io(error.to_string()))?,
            Some(serde_json::to_value(plan).map_err(|error| WikiError::Io(error.to_string()))?),
            None,
        ),
        Err(error) => (
            Value::String("blocked".to_owned()),
            None,
            Some(error.to_string()),
        ),
    };
    Ok(json!({
        "review_id": source.scan_metadata.as_ref().map(|metadata| &metadata.review_id),
        "source_claim_id": source.claim_id,
        "expected_source_digest": source.digest,
        "normalized_fact": source.normalized_fact,
        "kind": source.kind,
        "status": source.status,
        "provenance": source.provenance,
        "redaction": {
            "performed": false,
            "reason": "promotion preserves the exact secret-screened reviewed fact",
        },
        "deduplication": {
            "decision": decision,
            "duplicate_claim_ids": duplicates,
        },
        "contradiction": {
            "claim_ids": contradictions,
            "blocked_reason": blocked_reason,
        },
        "replacement": {
            "claim_ids": request.supersedes,
            "automatic": false,
        },
        "request": request,
        "plan": plan_value,
    }))
}

fn build_scan_promotion_request(source: &CanonicalClaim) -> Result<RememberRequest, WikiError> {
    let key_material = serde_json::to_vec(&(
        "reviewed-scan-promotion-v1",
        source.kind,
        source.normalized_fact.as_str(),
    ))
    .map_err(|error| WikiError::Io(error.to_string()))?;
    let key_digest = sha256_digest(&key_material);
    let key_suffix = key_digest
        .strip_prefix("sha256:")
        .ok_or_else(|| WikiError::Io("promotion digest lost its algorithm prefix".to_owned()))?;
    Ok(RememberRequest {
        collection_id: USER_ROOT_COLLECTION_ID.to_owned(),
        claim_key: format!("promoted.{key_suffix}"),
        claim_id: None,
        locator: format!(".hive/knowledge/Claims/user-root/pending-{key_suffix}.md"),
        kind: source.kind,
        status: source.status,
        visibility: RagVisibility::Shared,
        normalized_fact: source.normalized_fact.clone(),
        provenance: ClaimProvenance {
            source_kind: RememberSourceKind::ReviewedArtifact,
            summary: "Explicitly approved reviewed scan promotion".to_owned(),
            locator: source.locator.clone(),
            digest: source.provenance.digest.clone(),
        },
        sources: vec![source.locator.clone()],
        supersedes: Vec::new(),
        expected_active_digest: None,
        observed_at: source.observed_at.clone(),
        verified_at: source.verified_at.clone(),
    })
}

fn parse_scan_promotion_options(
    arguments: &[String],
) -> Result<(ValuedOptions<'_>, PromotionMode, bool), WikiError> {
    let allowed = [
        "--user-root",
        "--collection",
        "--review-id",
        "--expected-source-digest",
        "--output",
    ];
    let mut valued = Vec::new();
    let mut mode = None;
    let mut confirmed = false;
    let mut index = 0_usize;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--confirm-global-promotion" {
            if std::mem::replace(&mut confirmed, true) {
                return Err(WikiError::InvalidInput(
                    "duplicate promotion confirmation".to_owned(),
                ));
            }
            index += 1;
            continue;
        }
        if matches!(option, "--dry-run" | "--apply") {
            let candidate = if option == "--apply" {
                PromotionMode::Apply
            } else {
                PromotionMode::DryRun
            };
            if mode.replace(candidate).is_some() {
                return Err(WikiError::InvalidInput(
                    "promotion requires exactly one mode".to_owned(),
                ));
            }
            index += 1;
            continue;
        }
        if !allowed.contains(&option) {
            return Err(WikiError::InvalidInput(format!(
                "unknown knowledge option: {option}"
            )));
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| WikiError::InvalidInput(format!("missing value for {option}")))?;
        if valued.iter().any(|(existing, _)| *existing == option) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate knowledge option: {option}"
            )));
        }
        valued.push((option, value.as_str()));
        index += 2;
    }
    let mode = mode.ok_or_else(|| {
        WikiError::InvalidInput("promotion requires --dry-run or --apply".to_owned())
    })?;
    if optional(&valued, "--output") != Some("json") {
        return Err(WikiError::InvalidInput(
            "knowledge commands require --output json".to_owned(),
        ));
    }
    if mode == PromotionMode::DryRun && confirmed {
        return Err(WikiError::InvalidInput(
            "promotion dry run does not accept approval confirmation".to_owned(),
        ));
    }
    if mode == PromotionMode::DryRun && optional(&valued, "--expected-source-digest").is_some() {
        return Err(WikiError::InvalidInput(
            "promotion dry run obtains the source digest from canonical state".to_owned(),
        ));
    }
    Ok((valued, mode, confirmed))
}

fn run_page_promote(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let (options, mode) = parse_promotion_options(arguments)?;
    let target = PathBuf::from(required(&options, "--target")?);
    let user_root = PathBuf::from(required(&options, "--user-root")?);
    let shared = shared_mutation_target(&target, &user_root, false)?;
    let _shared_operation_lock = (mode == PromotionMode::Apply)
        .then(|| acquire_shared_operation_lock(&shared))
        .transpose()?;
    let page_id = required(&options, "--page-id")?;
    let category = match required(&options, "--category")? {
        "fact" => PromotionCategory::Fact,
        "preference" => PromotionCategory::Preference,
        "workflow" => PromotionCategory::Workflow,
        _ => {
            return Err(WikiError::InvalidInput(
                "promotion category must be fact, preference, or workflow".to_owned(),
            ));
        }
    };
    let prepared = if mode == PromotionMode::Apply {
        prepare_shared_mutation(&shared)?
    } else {
        Vec::new()
    };
    let outcome = if mode == PromotionMode::Apply {
        promote_shared(&target, &user_root, page_id, category, mode)?
    } else {
        promote(&target, &user_root, page_id, category, mode)?
    };
    let changed_paths = outcome
        .changed_paths
        .iter()
        .cloned()
        .chain(prepared)
        .collect();
    let code = if mode == PromotionMode::Apply {
        "hive.knowledge-promoted"
    } else {
        "hive.knowledge-promotion-planned"
    };
    let message = if mode == PromotionMode::Apply {
        "project knowledge promoted into the canonical user-root store"
    } else {
        "knowledge promotion dry run completed without canonical mutation"
    };
    let digest = outcome.plan_digest.clone();
    let data = serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = if mode == PromotionMode::Apply {
        finish_shared_mutation(
            &user_root,
            Some(&SharedMutationTarget {
                user_root: user_root.clone(),
                target_kind: SharedTargetKind::UserRoot,
                namespace: "user-root".to_owned(),
            }),
            changed_paths,
            data,
            ".hive/knowledge",
        )?
    } else {
        (changed_paths, ".hive/knowledge", digest, data)
    };
    Ok(success(
        "PromoteKnowledge",
        code,
        message,
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

type ValuedOptions<'a> = Vec<(&'a str, &'a str)>;

fn parse_promotion_options(
    arguments: &[String],
) -> Result<(ValuedOptions<'_>, PromotionMode), WikiError> {
    parse_promotion_options_allowed(
        arguments,
        &[
            "--target",
            "--user-root",
            "--page-id",
            "--category",
            "--output",
        ],
    )
}

fn parse_promotion_options_allowed<'a>(
    arguments: &'a [String],
    allowed: &[&str],
) -> Result<(ValuedOptions<'a>, PromotionMode), WikiError> {
    let mut valued = Vec::new();
    let mut mode = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if matches!(option, "--dry-run" | "--apply") {
            let candidate = if option == "--apply" {
                PromotionMode::Apply
            } else {
                PromotionMode::DryRun
            };
            if mode.replace(candidate).is_some() {
                return Err(WikiError::InvalidInput(
                    "promotion requires exactly one mode".to_owned(),
                ));
            }
            index += 1;
            continue;
        }
        if !allowed.contains(&option) {
            return Err(WikiError::InvalidInput(format!(
                "unknown knowledge option: {option}"
            )));
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| WikiError::InvalidInput(format!("missing value for {option}")))?;
        if valued.iter().any(|(existing, _)| *existing == option) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate knowledge option: {option}"
            )));
        }
        valued.push((option, value.as_str()));
        index += 2;
    }
    if optional(&valued, "--output") != Some("json") {
        return Err(WikiError::InvalidInput(
            "knowledge commands require --output json".to_owned(),
        ));
    }
    Ok((
        valued,
        mode.ok_or_else(|| {
            WikiError::InvalidInput("promotion requires --dry-run or --apply".to_owned())
        })?,
    ))
}

fn run_lint(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(arguments, &["--target", "--user-root"])?;
    let target = PathBuf::from(required(&options, "--target")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_lint_target(&target, Path::new(root)))
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let lint_target = shared
        .as_ref()
        .filter(|shared| shared.target_kind == SharedTargetKind::UserRoot)
        .map_or(target.as_path(), |shared| shared.user_root.as_path());
    let mut issues = lint(lint_target)?;
    let shared_digest = if let Some(shared) = &shared {
        issues
            .retain(|issue| issue.code != "stale-index" || issue.locator != SHARED_INDEX_RELATIVE);
        match validate_shared_index(&shared.user_root) {
            Ok(digest) => Some(digest),
            Err(error) => {
                issues.push(LintIssue {
                    code: "stale-index".to_owned(),
                    severity: LintSeverity::Error,
                    locator: SHARED_INDEX_RELATIVE.to_owned(),
                    message: error.to_string(),
                });
                None
            }
        }
    } else {
        None
    };
    issues.sort_by(|left, right| {
        (&left.code, &left.locator, &left.message).cmp(&(
            &right.code,
            &right.locator,
            &right.message,
        ))
    });
    let has_error = issues
        .iter()
        .any(|issue| issue.severity == LintSeverity::Error);
    let data = json!({
        "issues": issues,
        "error_count": issue_count(&issues, true),
        "warning_count": issue_count(&issues, false),
        "shared_index_digest": shared_digest.clone(),
        "shared_index_path": shared.as_ref().map(|_| SHARED_INDEX_RELATIVE)
    });
    let report_digest = sha256_digest(
        &serde_json::to_vec(&data).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    let digest = shared_digest.as_deref().unwrap_or(&report_digest);
    let locator = if shared.is_some() {
        SHARED_INDEX_RELATIVE
    } else {
        ".hive/knowledge"
    };
    if has_error {
        Ok(KnowledgeResult {
            schema_version: 1,
            action: "LintKnowledge",
            status: "verification-failed",
            exit_code: 5,
            code: "hive.knowledge-lint-failed",
            message: "knowledge lint found canonical or derived-state errors".to_owned(),
            changed_paths: Vec::new(),
            evidence: vec![KnowledgeEvidence {
                kind: "report",
                locator: locator.to_owned(),
                digest: digest.to_owned(),
            }],
            next_action: Some("repair reported issues and run knowledge lint again".to_owned()),
            data: Some(data),
        })
    } else {
        Ok(success(
            "LintKnowledge",
            "hive.knowledge-lint-passed",
            "knowledge lint completed without errors",
            Vec::new(),
            locator,
            digest,
            data,
        ))
    }
}

fn issue_count(issues: &[LintIssue], errors: bool) -> usize {
    issues
        .iter()
        .filter(|issue| (issue.severity == LintSeverity::Error) == errors)
        .count()
}

fn run_delete(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--page-id",
            "--reason",
            "--replacement",
            "--timestamp",
            "--user-root",
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_mutation_target(&target, Path::new(root), true))
        .transpose()?;
    let _shared_operation_lock = shared
        .as_ref()
        .map(acquire_shared_operation_lock)
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let prepared = shared
        .as_ref()
        .map(prepare_shared_mutation)
        .transpose()?
        .unwrap_or_default();
    let outcome = if let Some(shared) = &shared {
        delete_page_shared(
            &target,
            required(&options, "--page-id")?,
            required(&options, "--reason")?,
            optional(&options, "--replacement"),
            required(&options, "--timestamp")?,
            &shared.user_root,
            &shared.namespace,
        )?
    } else {
        delete_page(
            &target,
            required(&options, "--page-id")?,
            required(&options, "--reason")?,
            optional(&options, "--replacement"),
            required(&options, "--timestamp")?,
        )?
    };
    let mutation =
        serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = finish_shared_mutation(
        &target,
        shared.as_ref(),
        outcome.changed_paths.into_iter().chain(prepared).collect(),
        mutation,
        ".hive/knowledge/suppression.yml",
    )?;
    Ok(success(
        "DeleteKnowledge",
        "hive.knowledge-deleted",
        "active Wiki page deleted and minimal suppression metadata recorded",
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

fn run_suppress(arguments: &[String]) -> Result<KnowledgeResult, WikiError> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--fingerprint",
            "--source-locator",
            "--reason",
            "--replacement",
            "--timestamp",
            "--user-root",
        ],
    )?;
    let target = PathBuf::from(required(&options, "--target")?);
    let shared = optional(&options, "--user-root")
        .map(|root| shared_mutation_target(&target, Path::new(root), true))
        .transpose()?;
    let _shared_operation_lock = shared
        .as_ref()
        .map(acquire_shared_operation_lock)
        .transpose()?;
    if shared.is_none() {
        authorize_legacy_target(&target)?;
    }
    let entry = SuppressionEntry {
        fingerprint: required(&options, "--fingerprint")?.to_owned(),
        source_locator: required(&options, "--source-locator")?.to_owned(),
        reason: required(&options, "--reason")?.to_owned(),
        replacement: optional(&options, "--replacement").map(ToOwned::to_owned),
        timestamp: required(&options, "--timestamp")?.to_owned(),
    };
    let prepared = shared
        .as_ref()
        .map(prepare_shared_mutation)
        .transpose()?
        .unwrap_or_default();
    let outcome = if let Some(shared) = &shared {
        suppress_shared(&target, entry, &shared.user_root, &shared.namespace)?
    } else {
        suppress(&target, entry)?
    };
    let mutation =
        serde_json::to_value(&outcome).map_err(|error| WikiError::Io(error.to_string()))?;
    let (changed_paths, locator, digest, data) = finish_shared_mutation(
        &target,
        shared.as_ref(),
        outcome.changed_paths.into_iter().chain(prepared).collect(),
        mutation,
        ".hive/knowledge/suppression.yml",
    )?;
    Ok(success(
        "SuppressKnowledge",
        "hive.knowledge-suppressed",
        "minimal source fingerprint suppression recorded",
        changed_paths,
        locator,
        &digest,
        data,
    ))
}

#[derive(Debug, Eq, PartialEq)]
enum SharedTargetKind {
    UserRoot,
    RegisteredProject,
}

#[derive(Debug)]
struct SharedMutationTarget {
    user_root: PathBuf,
    target_kind: SharedTargetKind,
    namespace: String,
}

fn shared_mutation_target(
    target: &Path,
    user_root: &Path,
    allow_user_root: bool,
) -> Result<SharedMutationTarget, WikiError> {
    ensure_consumer_target(target).map_err(|error| WikiError::Conflict(error.to_string()))?;
    require_shared_wiki_enabled(user_root)?;
    let canonical_user = hive_wiki::shared::canonical_root(user_root)?;
    let canonical_target = hive_wiki::shared::canonical_root(target)?;
    let registry = load_project_registry(&canonical_user)?;
    let (target_kind, namespace) = if canonical_target == canonical_user {
        if !allow_user_root {
            return Err(WikiError::InvalidInput(
                "knowledge target must be an enabled registered project".to_owned(),
            ));
        }
        (SharedTargetKind::UserRoot, "user-root".to_owned())
    } else if let Some(project) = registry.projects.iter().find(|project| {
        project.enabled
            && hive_wiki::shared::canonical_root(&project.root)
                .is_ok_and(|registered| registered == canonical_target)
    }) {
        (SharedTargetKind::RegisteredProject, project.id.clone())
    } else {
        return Err(WikiError::InvalidInput(
            "knowledge target is not enabled in the project registry".to_owned(),
        ));
    };
    Ok(SharedMutationTarget {
        user_root: canonical_user,
        target_kind,
        namespace,
    })
}

fn shared_lint_target(target: &Path, user_root: &Path) -> Result<SharedMutationTarget, WikiError> {
    ensure_consumer_target(target).map_err(|error| WikiError::Conflict(error.to_string()))?;
    require_shared_wiki_enabled(user_root)?;
    let canonical_user = hive_wiki::shared::canonical_root(user_root)?;
    let canonical_target = hive_wiki::shared::canonical_root(target)?;
    let registry = load_project_registry(&canonical_user)?;
    let registered = registry.projects.iter().find(|project| {
        project.enabled
            && hive_wiki::shared::canonical_root(&project.root)
                .is_ok_and(|registered| registered == canonical_target)
    });
    let (target_kind, namespace) = if canonical_target == canonical_user {
        (SharedTargetKind::UserRoot, "user-root".to_owned())
    } else if let Some(project) = registered {
        (SharedTargetKind::RegisteredProject, project.id.clone())
    } else {
        (SharedTargetKind::UserRoot, "user-root".to_owned())
    };
    Ok(SharedMutationTarget {
        user_root: canonical_user,
        target_kind,
        namespace,
    })
}

fn prepare_shared_mutation(shared: &SharedMutationTarget) -> Result<Vec<String>, WikiError> {
    Ok(rebuild_shared_index(&shared.user_root)?
        .changed_paths
        .into_iter()
        .map(|path| format!("user-root:{path}"))
        .collect())
}

fn acquire_shared_operation_lock(
    shared: &SharedMutationTarget,
) -> Result<SharedKnowledgeOperationLock, WikiError> {
    RagStore::open(&shared.user_root)?.acquire_shared_operation_lock()
}

fn require_shared_wiki_enabled(user_root: &Path) -> Result<(), WikiError> {
    match super::user_setup::operational_wiki_preferences(user_root) {
        Ok(preferences) if preferences.enabled && preferences.backend == super::user_setup::WikiBackend::Markdown => Ok(()),
        Ok(preferences) if !preferences.enabled => Err(WikiError::Conflict(
            "global Wiki is disabled; canonical Markdown is preserved and shared knowledge operations are unavailable"
                .to_owned(),
        )),
        Ok(_) => Err(WikiError::Conflict(
            "global Wiki backend notion has no local Markdown canonical store; use knowledge notion"
                .to_owned(),
        )),
        Err(error) => Err(WikiError::Verification(format!(
            "cannot authorize shared knowledge operation: {error}"
        ))),
    }
}

fn authorize_legacy_target(target: &Path) -> Result<(), WikiError> {
    super::project_upgrade::authenticate_legacy_knowledge_target(target).map_err(|error| {
        WikiError::Verification(format!(
            "legacy project-local knowledge route is not authenticated: {error}"
        ))
    })
}

fn finish_shared_mutation(
    target: &Path,
    shared: Option<&SharedMutationTarget>,
    mut changed_paths: Vec<String>,
    mutation: Value,
    legacy_locator: &'static str,
) -> Result<(Vec<String>, &'static str, String, Value), WikiError> {
    let Some(shared) = shared else {
        let digest = mutation
            .get("logical_digest")
            .and_then(Value::as_str)
            .ok_or_else(|| WikiError::Io("mutation digest is missing".to_owned()))?
            .to_owned();
        return Ok((changed_paths, legacy_locator, digest, mutation));
    };
    let removed_legacy_project_indexes =
        if shared.target_kind == SharedTargetKind::RegisteredProject {
            remove_legacy_project_indexes(target)?
        } else {
            Vec::new()
        };
    changed_paths.retain(|path| !LEGACY_DERIVED_RELATIVES.contains(&path.as_str()));
    let rebuilt = rebuild_shared_index(&shared.user_root)?;
    changed_paths.extend(
        rebuilt
            .changed_paths
            .iter()
            .map(|path| format!("user-root:{path}")),
    );
    let digest = rebuilt.logical_digest.clone();
    let mut data = mutation;
    let data_object = data
        .as_object_mut()
        .ok_or_else(|| WikiError::Io("mutation result is not an object".to_owned()))?;
    data_object.insert(
        "shared_index".to_owned(),
        serde_json::to_value(&rebuilt).map_err(|error| WikiError::Io(error.to_string()))?,
    );
    data_object.insert(
        "removed_legacy_project_indexes".to_owned(),
        serde_json::to_value(removed_legacy_project_indexes)
            .map_err(|error| WikiError::Io(error.to_string()))?,
    );
    Ok((changed_paths, SHARED_INDEX_RELATIVE, digest, data))
}

fn remove_legacy_project_indexes(target: &Path) -> Result<Vec<String>, WikiError> {
    let mut existing = Vec::new();
    for relative in LEGACY_DERIVED_RELATIVES {
        ensure_no_symlink_ancestors(target, Path::new(relative))
            .map_err(|error| WikiError::Conflict(error.to_string()))?;
        let path = target.join(relative);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(WikiError::Conflict(format!(
                    "legacy project index path is not a regular no-follow file: {relative}"
                )));
            }
            Ok(_) => existing.push((relative, path)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(WikiError::Io(format!(
                    "cannot inspect legacy project index {relative}: {error}"
                )));
            }
        }
    }
    let mut removed = Vec::new();
    for (relative, path) in existing {
        std::fs::remove_file(&path).map_err(|error| {
            WikiError::Io(format!(
                "cannot remove legacy project index {relative}: {error}"
            ))
        })?;
        removed.push(relative.to_owned());
    }
    Ok(removed)
}

fn parse_options<'a>(
    arguments: &'a [String],
    allowed: &[&str],
) -> Result<Vec<(&'a str, &'a str)>, WikiError> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--output" {
            let value = arguments
                .get(index + 1)
                .ok_or_else(|| WikiError::InvalidInput("missing value for --output".to_owned()))?;
            if value != "json" {
                return Err(WikiError::InvalidInput(
                    "knowledge commands require --output json".to_owned(),
                ));
            }
            output.push((option, value.as_str()));
            index += 2;
            continue;
        }
        if !allowed.contains(&option) {
            return Err(WikiError::InvalidInput(format!(
                "unknown knowledge option: {option}"
            )));
        }
        let value = arguments
            .get(index + 1)
            .ok_or_else(|| WikiError::InvalidInput(format!("missing value for {option}")))?;
        if output.iter().any(|(existing, _)| *existing == option) {
            return Err(WikiError::InvalidInput(format!(
                "duplicate knowledge option: {option}"
            )));
        }
        output.push((option, value.as_str()));
        index += 2;
    }
    if optional(&output, "--output") != Some("json") {
        return Err(WikiError::InvalidInput(
            "knowledge commands require --output json".to_owned(),
        ));
    }
    Ok(output)
}

fn required<'a>(options: &[(&'a str, &'a str)], name: &str) -> Result<&'a str, WikiError> {
    optional(options, name)
        .ok_or_else(|| WikiError::InvalidInput(format!("missing required option {name}")))
}

fn optional<'a>(options: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    options
        .iter()
        .find_map(|(option, value)| (*option == name).then_some(*value))
}

fn success(
    action: &'static str,
    code: &'static str,
    message: &str,
    mut changed_paths: Vec<String>,
    locator: &str,
    digest: &str,
    data: Value,
) -> KnowledgeResult {
    changed_paths.sort();
    changed_paths.dedup();
    KnowledgeResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths,
        evidence: vec![KnowledgeEvidence {
            kind: "report",
            locator: locator.to_owned(),
            digest: digest.to_owned(),
        }],
        next_action: None,
        data: Some(data),
    }
}

fn failure(action: &'static str, error: &WikiError) -> KnowledgeResult {
    KnowledgeResult {
        schema_version: 1,
        action,
        status: error.status(),
        exit_code: error.exit_code(),
        code: error.code(),
        message: error.to_string(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

fn emit(result: &KnowledgeResult) {
    match serde_json::to_string(result) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            println!("{{\"schema_version\":1,\"action\":\"UnknownAction\",\"status\":\"error\",\"exit_code\":10,\"code\":\"hive.internal-error\",\"message\":\"JSON serialization failed\",\"changed_paths\":[],\"evidence\":[],\"next_action\":null}}");
            eprintln!("error: {error}");
        }
    }
    if result.exit_code != 0 {
        eprintln!("error: {}", result.message);
    }
}

fn is_help(arguments: &[String]) -> bool {
    matches!(arguments, [argument] if argument == "-h" || argument == "--help")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hive_wiki::shared::{
        register_project, KnowledgeLanguage, KnowledgeVisibility, RegisteredProject,
    };
    use std::fs;
    use tempfile::TempDir;

    fn temp_root() -> TempDir {
        TempDir::new_in(std::env::current_dir().expect("current directory"))
            .expect("temporary root")
    }

    fn temp_root_outside_repository() -> TempDir {
        tempfile::tempdir().expect("external temporary root")
    }

    fn write_empty_knowledge(root: &Path) {
        fs::create_dir_all(root.join(".hive/knowledge/Wiki")).expect("Wiki directory");
        fs::create_dir_all(root.join(".hive/knowledge/Raw")).expect("Raw directory");
        fs::write(
            root.join(".hive/knowledge/suppression.yml"),
            "schema_version: 1\nentries: []\n",
        )
        .expect("suppression ledger");
    }

    fn wiki_draft(id: &str, kind: &str, body: &str) -> String {
        format!(
            "---\nschema_version: 1\nid: {id}\nkind: {kind}\nsummary: {id} summary\ntags: [shared-test]\naliases: []\nsources: [raw:self]\nlinks: []\ncontradictions: []\nstatus: active\ncreated_at: 2026-08-01T00:00:00Z\nupdated_at: 2026-08-01T00:00:00Z\n---\n\n{body}\n"
        )
    }

    fn add_arguments(
        project: &Path,
        user: &Path,
        source: &Path,
        draft: &Path,
        quick: bool,
    ) -> Vec<String> {
        let mut arguments = vec![
            "--target".to_owned(),
            project.to_string_lossy().into_owned(),
            "--source".to_owned(),
            source.to_string_lossy().into_owned(),
            "--wiki".to_owned(),
            draft.to_string_lossy().into_owned(),
            "--user-root".to_owned(),
            user.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        if quick {
            arguments.push("--quick".to_owned());
        }
        arguments
    }

    fn write_user_setup(root: &Path, wiki_enabled: bool) {
        fs::create_dir_all(root.join(".hive/config")).expect("user config");
        fs::write(
            root.join(".hive/config/user-setup.yml"),
            format!(
                "schema_version: 1\ninterface_language: en\nwiki:\n  enabled: {wiki_enabled}\n  language: both\nprofile:\n  id: web-developer\npersona:\n  id: balanced\nselected_hosts:\n  - codex\nskills:\n  mode: individual\n  selected:\n    - setup-hive\nusage_guard:\n  enabled: false\n  stop_remaining_percent: 20\n  codexbar_fallback_enabled: false\n"
            ),
        )
        .expect("user setup");
    }

    #[test]
    fn v09_knowledge_help_does_not_advertise_notion() {
        assert!(!KNOWLEDGE_USAGE.contains("notion"));
    }

    #[cfg(feature = "notion-preview")]
    fn write_notion_user_setup(root: &Path) {
        fs::create_dir_all(root.join(".hive/config")).expect("user config");
        fs::write(
            root.join(".hive/config/user-setup.yml"),
            "schema_version: 1\ninterface_language: en\nwiki:\n  enabled: true\n  language: both\n  backend: notion\n  notion:\n    workspace_id: workspace-a\n    scope_id: scope-a\n    local_index_consent: true\nprofile:\n  id: web-developer\npersona:\n  id: balanced\nselected_hosts:\n  - codex\nskills:\n  mode: individual\n  selected:\n    - setup-hive\nusage_guard:\n  enabled: false\n  stop_remaining_percent: 20\n  codexbar_fallback_enabled: false\n",
        )
        .expect("Notion user setup");
    }

    #[cfg(feature = "notion-preview")]
    fn write_notion_inputs(root: &Path) -> (PathBuf, PathBuf) {
        use hive_wiki::notion::{
            NotionAdapter, NotionCapabilityReceipt, NotionInventoryEntry, NotionPage,
            NotionSyncRequest, RequiredCapability,
        };

        let capability = root.join("notion-capability.json");
        let snapshot = root.join("notion-snapshot.json");
        fs::write(
            &capability,
            serde_json::to_vec(&NotionCapabilityReceipt {
                schema_version: 1,
                adapter: NotionAdapter::HostPlugin,
                workspace_id: "workspace-a".to_owned(),
                scope_id: "scope-a".to_owned(),
                capabilities: RequiredCapability::ALL.to_vec(),
                rest_consent: false,
            })
            .expect("capability JSON"),
        )
        .expect("write capability");
        fs::write(
            &snapshot,
            serde_json::to_vec(&NotionSyncRequest {
                schema_version: 1,
                workspace_id: "workspace-a".to_owned(),
                scope_id: "scope-a".to_owned(),
                inventory_complete: true,
                next_cursor: None,
                inventory: vec![NotionInventoryEntry {
                    page_id: "page-a".to_owned(),
                    revision: "rev-1".to_owned(),
                    deleted: false,
                }],
                pages: vec![NotionPage {
                    page_id: "page-a".to_owned(),
                    revision: "rev-1".to_owned(),
                    title: "Deployment guide".to_owned(),
                    body: "Alpha deployment procedure".to_owned(),
                    kind: "workflow".to_owned(),
                    language: "en".to_owned(),
                    tags: vec!["deployment".to_owned()],
                    aliases: vec!["ship".to_owned()],
                    sources: Vec::new(),
                    complete: true,
                    truncated: false,
                    unknown_blocks: Vec::new(),
                }],
            })
            .expect("snapshot JSON"),
        )
        .expect("write snapshot");
        (capability, snapshot)
    }

    #[cfg(feature = "notion-preview")]
    #[test]
    fn v09_notion_retrieval_is_rejected_without_local_wiki_markdown() {
        let user = temp_root();
        write_notion_user_setup(user.path());
        let (capability, snapshot) = write_notion_inputs(user.path());
        let result = run_notion_retrieve(&[
            "--user-root".to_owned(),
            user.path().display().to_string(),
            "--capability".to_owned(),
            capability.display().to_string(),
            "--snapshot".to_owned(),
            snapshot.display().to_string(),
            "--query".to_owned(),
            "Alpha".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ]);
        let Err(error) = result else {
            panic!("v0.9 must reject Notion user setup");
        };
        match error {
            WikiError::Verification(message) => {
                assert!(message.contains("installed user setup is invalid"));
                assert!(message.contains("\"markdown\" was expected"));
            }
            other => panic!("unexpected Notion rejection: {other:?}"),
        }
        assert!(!user.path().join(".hive/knowledge/Wiki").exists());
        assert!(!user.path().join(NOTION_LEDGER_RELATIVE).exists());
    }

    fn write_remember_request(root: &Path) -> PathBuf {
        use hive_wiki::rag::{
            AssertionStatus, ClaimKind, ClaimProvenance, RagVisibility, RememberSourceKind,
        };

        let path = root.join("remember-request.json");
        let request = RememberRequest {
            collection_id: "user-root".to_owned(),
            claim_key: "preference.commit-guidance".to_owned(),
            claim_id: None,
            locator: "auto".to_owned(),
            kind: ClaimKind::Preference,
            status: AssertionStatus::UserStated,
            visibility: RagVisibility::Shared,
            normalized_fact: "Prefer concise reusable commit guidance.".to_owned(),
            provenance: ClaimProvenance {
                source_kind: RememberSourceKind::UserStatement,
                summary: "Reviewed durable user preference".to_owned(),
                locator: "request:commit-guidance".to_owned(),
                digest: sha256_digest(b"reviewed durable user preference"),
            },
            sources: vec!["request:commit-guidance".to_owned()],
            supersedes: Vec::new(),
            expected_active_digest: None,
            observed_at: None,
            verified_at: None,
        };
        fs::write(&path, serde_json::to_vec(&request).expect("remember JSON"))
            .expect("remember request");
        path
    }

    fn write_collection_claim_request(
        root: &Path,
        file_name: &str,
        collection_id: &str,
        fact: &str,
        visibility: hive_wiki::rag::RagVisibility,
    ) -> PathBuf {
        use hive_wiki::rag::{AssertionStatus, ClaimKind, ClaimProvenance, RememberSourceKind};

        let path = root.join(file_name);
        let request = RememberRequest {
            collection_id: collection_id.to_owned(),
            claim_key: format!("test.{file_name}"),
            claim_id: None,
            locator: "auto".to_owned(),
            kind: ClaimKind::Decision,
            status: AssertionStatus::Verified,
            visibility,
            normalized_fact: fact.to_owned(),
            provenance: ClaimProvenance {
                source_kind: RememberSourceKind::ReviewedArtifact,
                summary: "Reviewed authorization fixture".to_owned(),
                locator: format!("fixture:{file_name}"),
                digest: sha256_digest(format!("fixture:{file_name}").as_bytes()),
            },
            sources: vec![format!("fixture:{file_name}")],
            supersedes: Vec::new(),
            expected_active_digest: None,
            observed_at: None,
            verified_at: Some("2026-08-01T00:00:00Z".to_owned()),
        };
        fs::write(&path, serde_json::to_vec(&request).expect("remember JSON"))
            .expect("remember request");
        path
    }

    fn auto_retrieval_hits(user_root: &Path, target: &Path, query: &str) -> Vec<Value> {
        run_retrieve(&[
            "--user-root".to_owned(),
            user_root.to_string_lossy().into_owned(),
            "--target".to_owned(),
            target.to_string_lossy().into_owned(),
            "--scope".to_owned(),
            "auto".to_owned(),
            "--query".to_owned(),
            query.to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("automatic retrieval")
        .data
        .expect("retrieval data")["hits"]
            .as_array()
            .expect("hits")
            .clone()
    }

    fn registered_roots(enabled: bool) -> (TempDir, TempDir) {
        let user = temp_root();
        let project = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        write_empty_knowledge(project.path());
        register_project(
            user.path(),
            RegisteredProject {
                id: "project-test".to_owned(),
                root: hive_wiki::shared::canonical_root(project.path()).expect("canonical project"),
                enabled,
                language: KnowledgeLanguage::En,
                visibility: KnowledgeVisibility::ProjectPrivate,
            },
        )
        .expect("register project");
        (user, project)
    }

    #[test]
    fn disabled_global_wiki_blocks_shared_operations_and_preserves_markdown() {
        let (user, project) = registered_roots(true);
        write_user_setup(user.path(), false);
        let page = project.path().join(".hive/knowledge/Wiki/example.md");
        fs::write(&page, b"canonical markdown\n").expect("canonical page");

        let error = shared_mutation_target(project.path(), user.path(), true)
            .expect_err("disabled Wiki must block shared mutation");
        assert_eq!(error.code(), "hive.knowledge-conflict");
        assert!(error.to_string().contains("global Wiki is disabled"));
        let Err(query_error) = run_shared_query(
            project.path(),
            user.path(),
            Some("canonical"),
            None,
            None,
            20,
        ) else {
            panic!("disabled Wiki must block shared query");
        };
        assert_eq!(query_error.code(), "hive.knowledge-conflict");
        let rebuild_arguments = vec![
            "rebuild".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        assert_ne!(run_index(&rebuild_arguments), ExitCode::SUCCESS);
        assert_eq!(
            fs::read(&page).expect("preserved canonical page"),
            b"canonical markdown\n"
        );
        assert!(!user.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn explicit_user_root_missing_registry_never_falls_back_to_project_index() {
        let user = temp_root();
        let project = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        write_empty_knowledge(project.path());
        let arguments = vec![
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--text".to_owned(),
            "missing".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let Err(error) = run_query(&arguments) else {
            panic!("missing registry must fail closed");
        };

        assert_eq!(error.code(), "hive.knowledge-io-error");
        assert!(error.to_string().contains("cannot open project registry"));
        assert!(!user.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn unauthenticated_legacy_target_never_creates_a_project_index() {
        let project = temp_root();
        write_empty_knowledge(project.path());
        let arguments = vec![
            "rebuild".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        assert_ne!(run_index(&arguments), ExitCode::SUCCESS);
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn shared_mutations_require_the_enabled_registered_target() {
        let (user, enabled) = registered_roots(true);
        let disabled = temp_root();
        let unregistered = temp_root();
        register_project(
            user.path(),
            RegisteredProject {
                id: "project-disabled".to_owned(),
                root: hive_wiki::shared::canonical_root(disabled.path())
                    .expect("canonical disabled"),
                enabled: false,
                language: KnowledgeLanguage::Ko,
                visibility: KnowledgeVisibility::ProjectPrivate,
            },
        )
        .expect("register disabled project");

        assert_eq!(
            shared_mutation_target(enabled.path(), user.path(), false)
                .expect("enabled project")
                .target_kind,
            SharedTargetKind::RegisteredProject
        );
        assert_eq!(
            shared_mutation_target(user.path(), user.path(), true)
                .expect("exact user root")
                .target_kind,
            SharedTargetKind::UserRoot
        );
        assert!(shared_mutation_target(user.path(), user.path(), false).is_err());
        assert!(shared_mutation_target(disabled.path(), user.path(), true).is_err());
        assert!(shared_mutation_target(unregistered.path(), user.path(), true).is_err());
    }

    #[test]
    fn shared_mutation_removes_only_fixed_project_indexes_and_rebuilds_root() {
        let (user, project) = registered_roots(true);
        fs::create_dir_all(project.path().join(".hive/index")).expect("project index directory");
        for relative in LEGACY_DERIVED_RELATIVES {
            fs::write(project.path().join(relative), b"legacy").expect("legacy index artifact");
        }
        let shared =
            shared_mutation_target(project.path(), user.path(), true).expect("registered project");
        let (changed_paths, locator, digest, data) = finish_shared_mutation(
            project.path(),
            Some(&shared),
            vec![
                ".hive/knowledge/Wiki/example.md".to_owned(),
                SHARED_INDEX_RELATIVE.to_owned(),
            ],
            json!({"logical_digest": "sha256:legacy"}),
            ".hive/knowledge",
        )
        .expect("finish shared mutation");

        assert_eq!(locator, SHARED_INDEX_RELATIVE);
        assert!(digest.starts_with("sha256:"));
        assert!(user.path().join(SHARED_INDEX_RELATIVE).is_file());
        for relative in LEGACY_DERIVED_RELATIVES {
            assert!(!project.path().join(relative).exists());
        }
        assert_eq!(
            changed_paths,
            vec![
                ".hive/knowledge/Wiki/example.md",
                "user-root:.hive/config/collections.yml",
                "user-root:.hive/config/rag-trust.json",
                "user-root:.hive/index/hive.sqlite3",
                "user-root:.hive/index/rag-generation.json"
            ]
        );
        assert_eq!(
            data["shared_index"]["logical_digest"],
            Value::String(digest)
        );
        assert_eq!(data["logical_digest"], "sha256:legacy");
        assert!(data.get("mutation").is_none());
    }

    #[test]
    fn quick_add_keeps_reviewed_provenance_secret_gate_and_single_rag_index() {
        let (user, project) = registered_roots(true);
        let source = project.path().join("source.md");
        let draft = project.path().join("draft.md");
        fs::write(&source, "Reviewed deployment convention.").expect("source");
        fs::write(
            &draft,
            wiki_draft(
                "deployment-convention",
                "concept",
                "Use reviewed deployment checks.",
            ),
        )
        .expect("draft");

        let added = run_add(&add_arguments(
            project.path(),
            user.path(),
            &source,
            &draft,
            true,
        ))
        .expect("quick add");

        assert_eq!(added.action, "AddKnowledge");
        assert_eq!(added.code, "hive.knowledge-added");
        assert_eq!(added.data.as_ref().expect("data")["quick"], true);
        let page_path = project
            .path()
            .join(".hive/knowledge/Wiki/deployment-convention.md");
        let page = hive_wiki::parse_page_bytes(
            &fs::read(&page_path).expect("canonical page"),
            "deployment-convention",
        )
        .expect("canonical page");
        assert_eq!(page.frontmatter.sources.len(), 1);
        assert!(page.frontmatter.sources[0].starts_with("raw:.hive/knowledge/Raw/"));
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!project.path().join(".hive/index/.stale").exists());
        RagStore::open(user.path())
            .expect("store")
            .validate_current()
            .expect("RAG schema");

        let secret_source = project.path().join("secret-source.md");
        let secret_draft = project.path().join("secret-draft.md");
        fs::write(
            &secret_source,
            "OPENAI_API_KEY=sk-proj-1234567890abcdefghijklmnop",
        )
        .expect("secret source");
        fs::write(
            &secret_draft,
            wiki_draft("blocked-secret", "concept", "Must never be stored."),
        )
        .expect("secret draft");
        let Err(error) = run_add(&add_arguments(
            project.path(),
            user.path(),
            &secret_source,
            &secret_draft,
            true,
        )) else {
            panic!("secret gate must reject quick add");
        };
        assert_eq!(error.code(), "hive.knowledge-invalid-input");
        assert!(!project
            .path()
            .join(".hive/knowledge/Wiki/blocked-secret.md")
            .exists());
        RagStore::open(user.path())
            .expect("store")
            .validate_current()
            .expect("RAG remains current");
    }

    #[test]
    #[allow(clippy::too_many_lines, clippy::useless_vec)]
    fn graph_preview_reads_canonical_pages_without_derived_write() {
        let (user, project) = registered_roots(true);
        let source = project.path().join("source.md");
        let draft = project.path().join("draft.md");
        fs::write(&source, "Graph preview source.").expect("source");
        fs::write(
            &draft,
            wiki_draft("graph-preview", "concept", "Graph preview body."),
        )
        .expect("draft");
        run_add(&add_arguments(
            project.path(),
            user.path(),
            &source,
            &draft,
            true,
        ))
        .expect("add page");
        let canonical_path = project.path().join(".hive/knowledge/Wiki/graph-preview.md");
        let canonical_before = fs::read(&canonical_path).expect("canonical page");

        let preview = run_graph(&vec![
            "preview".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--scope".to_owned(),
            "project".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("graph preview");
        assert_eq!(preview.code, "hive.knowledge-graph-preview");
        let data = preview.data.expect("preview data");
        assert_eq!(data["node_count"], 1);
        assert_eq!(data["writes"], false);

        let query = run_graph(&vec![
            "query".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--node-id".to_owned(),
            "graph-preview".to_owned(),
            "--text".to_owned(),
            "Graph preview".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("graph query");
        let query_data = query.data.expect("query data");
        let matches = query_data["matches"].as_array().expect("metadata matches");
        assert_eq!(matches.len(), 2);
        assert!(matches.iter().all(|edge| edge.get("body").is_none()));
        assert_eq!(
            query_data["matched_lanes"],
            json!(["fts", "markdown-graph"])
        );
        assert_eq!(
            query_data["fts"]["hits"]
                .as_array()
                .expect("FTS hits")
                .len(),
            1
        );
        assert_eq!(
            query_data["metadata"]
                .as_array()
                .expect("node metadata")
                .len(),
            1
        );

        let status = run_graph(&vec![
            "status".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("graph status");
        assert_eq!(status.data.expect("status data")["active"], false);

        let rebuilt = run_graph(&vec![
            "rebuild".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("graph rebuild");
        assert_eq!(rebuilt.action, "RebuildKnowledgeIndex");
        let rebuilt_data = rebuilt.data.expect("rebuild data");
        assert_eq!(rebuilt_data["writes"], true);
        assert!(project
            .path()
            .join(rebuilt_data["generation_path"].as_str().expect("path"))
            .is_file());

        let status = run_graph(&vec![
            "status".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("active graph status");
        assert_eq!(status.data.expect("active status data")["active"], true);

        for format in ["json", "html"] {
            let exported = run_graph(&[
                "export".to_owned(),
                "--target".to_owned(),
                project.path().to_string_lossy().into_owned(),
                "--format".to_owned(),
                format.to_owned(),
                "--output".to_owned(),
                "json".to_owned(),
            ])
            .expect("graph export");
            assert_eq!(exported.action, "ExportKnowledge");
            assert_eq!(exported.changed_paths.len(), 1);
            assert!(project.path().join(&exported.changed_paths[0]).is_file());
        }

        let disabled = run_graph(&vec![
            "disable".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("graph disable");
        assert_eq!(disabled.action, "DeleteKnowledge");
        assert_eq!(disabled.changed_paths.len(), 2);
        assert_eq!(
            fs::read(&canonical_path).expect("canonical page after graph disable"),
            canonical_before
        );
        let after = run_query(&[
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--text".to_owned(),
            "Graph preview".to_owned(),
            "--limit".to_owned(),
            "10".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("FTS after graph disable");
        assert_eq!(
            after.data.expect("FTS data")["hits"]
                .as_array()
                .expect("FTS hits")
                .len(),
            1
        );

        let Err(error) = run_graph(&vec![
            "preview".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--scope".to_owned(),
            "confidential".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ]) else {
            panic!("collection scope requires its separate authorization path");
        };
        assert!(error.to_string().contains("only project or source scope"));
    }

    #[cfg(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64", target_env = "musl"),
        all(target_os = "macos", target_arch = "aarch64")
    ))]
    #[test]
    #[allow(clippy::too_many_lines)]
    fn graphify_code_rebuild_requires_exact_receipt_and_falls_back_to_native() {
        let (user, project) = registered_roots(true);
        let source = project.path().join("source.md");
        let draft = project.path().join("draft.md");
        fs::write(&source, "Graphify fallback source.").expect("source");
        fs::write(
            &draft,
            wiki_draft("graphify-fallback", "concept", "Graphify fallback body."),
        )
        .expect("draft");
        run_add(&add_arguments(
            project.path(),
            user.path(),
            &source,
            &draft,
            true,
        ))
        .expect("add page");
        let input = br#"{"nodes":[{"id":"main","source_file":"src/main.rs","source_location":"L1"},{"id":"store","source_file":"src/store.rs","source_location":"L2"}],"edges":[{"source":"main","target":"store","relation":"calls","confidence":"EXTRACTED"}],"hyperedges":[],"input_tokens":0,"output_tokens":0}"#;
        let input_path = project.path().join("graphify-input.json");
        fs::write(&input_path, input).expect("Graphify input");
        let consent = graphify_consent("project").expect("Graphify consent");
        let receipt_path = project.path().join("graphify-receipt.json");
        fs::write(
            &receipt_path,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "package_version": GRAPHIFY_VERSION,
                "wheel_digest": GRAPHIFY_WHEEL_DIGEST,
                "dependency_lock_digest": graphify_dependency_lock_digest().expect("lock digest"),
                "executable_digest": format!("sha256:{}", "a1".repeat(32)),
                "python_identity_digest": format!("sha256:{}", "b2".repeat(32)),
                "consent_digest": consent.consent_digest.clone(),
                "source_commit": "0".repeat(40),
                "source_tree_digest": format!("sha256:{}", "c3".repeat(32)),
                "graph_input_digest": sha256_digest(input),
                "command": ["extract", "--force", "--code-only", "--no-cluster"],
                "provider_api_calls": 0,
                "api_keys_read": 0,
                "query_logs": 0,
                "watcher": false,
                "git_hooks": false,
                "mcp_registration": false,
                "network_requests": 0
            }))
            .expect("receipt JSON"),
        )
        .expect("receipt");
        let target = project.path().to_string_lossy().into_owned();
        let enabled = run_graph(&[
            "enable".to_owned(),
            "--target".to_owned(),
            target.clone(),
            "--engine".to_owned(),
            "graphify-code".to_owned(),
            "--consent-digest".to_owned(),
            consent.consent_digest.clone(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("Graphify enable");
        assert_eq!(enabled.code, "hive.knowledge-graph-enabled");
        assert_eq!(enabled.changed_paths, [PROJECT_GRAPHIFY_CONSENT_RELATIVE]);
        let rebuilt = run_graph(&[
            "rebuild".to_owned(),
            "--target".to_owned(),
            target.clone(),
            "--engine".to_owned(),
            "graphify-code".to_owned(),
            "--input".to_owned(),
            input_path.to_string_lossy().into_owned(),
            "--receipt".to_owned(),
            receipt_path.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("Graphify rebuild");
        assert_eq!(rebuilt.changed_paths.len(), 2);
        assert_eq!(rebuilt.data.expect("data")["engine"], "graphify-code");

        let queried = run_graph(&[
            "query".to_owned(),
            "--target".to_owned(),
            target.clone(),
            "--engine".to_owned(),
            "graphify-code".to_owned(),
            "--node-id".to_owned(),
            "main".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("Graphify query");
        let data = queried.data.expect("query data");
        assert_eq!(data["matches"].as_array().expect("matches").len(), 1);
        assert_eq!(data["fallback"], false);

        run_graph(&[
            "disable".to_owned(),
            "--target".to_owned(),
            target.clone(),
            "--engine".to_owned(),
            "graphify-code".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("Graphify disable");
        let fallback = run_graph(&[
            "query".to_owned(),
            "--target".to_owned(),
            target.clone(),
            "--engine".to_owned(),
            "graphify-code".to_owned(),
            "--node-id".to_owned(),
            "graphify-fallback".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("native fallback");
        assert_eq!(fallback.data.expect("fallback data")["fallback"], true);

        run_graph(&[
            "enable".to_owned(),
            "--target".to_owned(),
            target,
            "--engine".to_owned(),
            "graphify-code".to_owned(),
            "--consent-digest".to_owned(),
            consent.consent_digest,
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("Graphify re-enable");

        let mut tampered = fs::read(&receipt_path).expect("receipt bytes");
        tampered.extend_from_slice(b" ");
        fs::write(&receipt_path, tampered).expect("tamper receipt bytes");
        fs::write(&input_path, b"{}\n").expect("tamper input");
        let Err(error) = run_graph(&[
            "rebuild".to_owned(),
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--engine".to_owned(),
            "graphify-code".to_owned(),
            "--input".to_owned(),
            input_path.to_string_lossy().into_owned(),
            "--receipt".to_owned(),
            receipt_path.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ]) else {
            panic!("tampered input must fail");
        };
        assert!(matches!(error, WikiError::Verification(_)));
    }

    #[test]
    fn user_root_add_uses_the_rag_writer_without_nested_locking_or_schema_flip() {
        let user = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        ensure_project_registry(user.path()).expect("project registry");
        rebuild_shared_index(user.path()).expect("initial RAG");
        let source = user.path().join("root-source.md");
        let draft = user.path().join("root-draft.md");
        fs::write(&source, "Reviewed global commit convention.").expect("source");
        fs::write(
            &draft,
            wiki_draft(
                "global-commit-convention",
                "workflow",
                "Use the reviewed global commit convention.",
            ),
        )
        .expect("draft");

        run_add(&add_arguments(
            user.path(),
            user.path(),
            &source,
            &draft,
            true,
        ))
        .expect("user-root add");

        assert!(!user.path().join(".hive/index/.stale").exists());
        assert!(!user
            .path()
            .join(hive_wiki::store::RAG_DIRTY_RELATIVE)
            .exists());
        RagStore::open(user.path())
            .expect("store")
            .validate_current()
            .expect("RAG schema remains current");
        let queried = run_shared_query(
            user.path(),
            user.path(),
            Some("global commit"),
            None,
            None,
            20,
        )
        .expect("root query");
        assert_eq!(
            queried.data.expect("query data")["hits"][0]["page_id"],
            "global-commit-convention"
        );
    }

    #[test]
    fn shared_category_query_and_list_use_only_the_rag_facade() {
        let (user, project) = registered_roots(true);
        for (id, kind, body) in [
            ("deployment-concept", "concept", "deployment shared term"),
            ("deployment-workflow", "workflow", "deployment shared term"),
        ] {
            let source = project.path().join(format!("{id}-source.md"));
            let draft = project.path().join(format!("{id}-draft.md"));
            fs::write(&source, format!("Reviewed source for {id}.")).expect("source");
            fs::write(&draft, wiki_draft(id, kind, body)).expect("draft");
            run_add(&add_arguments(
                project.path(),
                user.path(),
                &source,
                &draft,
                false,
            ))
            .expect("shared add");
        }

        let category_only =
            run_shared_query(project.path(), user.path(), None, None, Some("concept"), 20)
                .expect("category query");
        assert_eq!(
            category_only.data.expect("query data")["hits"]
                .as_array()
                .expect("hits")
                .len(),
            1
        );
        let combined = run_shared_query(
            project.path(),
            user.path(),
            Some("deployment"),
            None,
            Some("workflow"),
            20,
        )
        .expect("combined query");
        assert_eq!(
            combined.data.expect("query data")["hits"][0]["page_id"],
            "deployment-workflow"
        );
        let list = run_list(&[
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("shared list");
        assert_eq!(
            list.data.expect("list data")["pages"]
                .as_array()
                .expect("pages")
                .len(),
            2
        );
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn shared_delete_and_suppress_never_create_a_project_sqlite_database() {
        let (user, project) = registered_roots(true);
        let source = project.path().join("delete-source.md");
        let draft = project.path().join("delete-draft.md");
        fs::write(&source, "Reviewed obsolete convention.").expect("source");
        fs::write(
            &draft,
            wiki_draft("obsolete-convention", "concept", "Obsolete convention."),
        )
        .expect("draft");
        run_add(&add_arguments(
            project.path(),
            user.path(),
            &source,
            &draft,
            false,
        ))
        .expect("shared add");

        let deleted = run_delete(&[
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--page-id".to_owned(),
            "obsolete-convention".to_owned(),
            "--reason".to_owned(),
            "user-request".to_owned(),
            "--timestamp".to_owned(),
            "2026-08-01T00:01:00Z".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("shared delete");
        assert_eq!(deleted.code, "hive.knowledge-deleted");
        assert!(!project
            .path()
            .join(".hive/knowledge/Wiki/obsolete-convention.md")
            .exists());

        let fingerprint = sha256_digest(b"reviewed external obsolete source");
        let suppressed = run_suppress(&[
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--fingerprint".to_owned(),
            fingerprint.clone(),
            "--source-locator".to_owned(),
            "external:obsolete-source".to_owned(),
            "--reason".to_owned(),
            "obsolete".to_owned(),
            "--timestamp".to_owned(),
            "2026-08-01T00:02:00Z".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("shared suppress");
        assert_eq!(suppressed.code, "hive.knowledge-suppressed");
        let ledger = fs::read_to_string(project.path().join(".hive/knowledge/suppression.yml"))
            .expect("suppression ledger");
        assert!(ledger.contains(&fingerprint));
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
        assert!(!project.path().join(".hive/index/.stale").exists());
        RagStore::open(user.path())
            .expect("store")
            .validate_current()
            .expect("RAG remains current");
    }

    #[test]
    fn interrupted_shared_mutation_is_fail_closed_until_canonical_rebuild() {
        let (user, project) = registered_roots(true);
        rebuild_shared_index(user.path()).expect("initial RAG");
        let store = RagStore::open(user.path()).expect("store");
        let intended = wiki_draft(
            "interrupted-page",
            "concept",
            "Recovered canonical knowledge after an interrupted write.",
        )
        .into_bytes();
        store
            .begin_external_canonical_mutation(&[(
                PathBuf::from("collections/project-test/.hive/knowledge/Wiki/interrupted-page.md"),
                intended.clone(),
            )])
            .expect("dirty journal");

        let Err(error) = run_shared_query(
            project.path(),
            user.path(),
            Some("anything"),
            None,
            None,
            20,
        ) else {
            panic!("dirty RAG query must fail closed");
        };
        assert_eq!(error.code(), "hive.knowledge-verification-failed");
        assert!(user
            .path()
            .join(hive_wiki::store::RAG_DIRTY_RELATIVE)
            .is_file());

        rebuild_shared_index(user.path())
            .expect_err("rebuild must not erase an unfinished exact canonical write");
        assert!(user
            .path()
            .join(hive_wiki::store::RAG_DIRTY_RELATIVE)
            .is_file());
        fs::write(
            project
                .path()
                .join(".hive/knowledge/Wiki/interrupted-page.md"),
            intended,
        )
        .expect("complete exact canonical write");
        rebuild_shared_index(user.path()).expect("canonical recovery rebuild");

        assert!(!user
            .path()
            .join(hive_wiki::store::RAG_DIRTY_RELATIVE)
            .exists());
        RagStore::open(user.path())
            .expect("store")
            .validate_current()
            .expect("recovered RAG");
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn shared_lint_uses_shared_freshness_without_requiring_a_project_index() {
        let (user, project) = registered_roots(true);
        let rebuilt = rebuild_shared_index(user.path()).expect("shared rebuild");
        let arguments = vec![
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let result = run_lint(&arguments).expect("shared lint");

        assert_eq!(result.status, "success");
        assert_eq!(result.evidence[0].locator, SHARED_INDEX_RELATIVE);
        assert_eq!(result.evidence[0].digest, rebuilt.logical_digest);
        assert!(!result.data.expect("lint data")["issues"]
            .as_array()
            .expect("issues")
            .iter()
            .any(|issue| issue["code"] == "stale-index"));
        assert!(!project.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn shared_lint_falls_back_to_user_root_for_an_unregistered_project() {
        let user = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        let project = temp_root();
        register_project(
            user.path(),
            RegisteredProject {
                id: "disabled-project".to_owned(),
                root: hive_wiki::shared::canonical_root(project.path()).expect("canonical project"),
                enabled: false,
                language: KnowledgeLanguage::En,
                visibility: KnowledgeVisibility::ProjectPrivate,
            },
        )
        .expect("disabled project registry entry");
        let rebuilt = rebuild_shared_index(user.path()).expect("shared rebuild");
        let arguments = vec![
            "--target".to_owned(),
            project.path().to_string_lossy().into_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let result = run_lint(&arguments).expect("unregistered shared lint");

        assert_eq!(result.status, "success");
        assert_eq!(result.evidence[0].locator, SHARED_INDEX_RELATIVE);
        assert_eq!(result.evidence[0].digest, rebuilt.logical_digest);
        assert!(!project.path().join(".hive").exists());
    }

    #[test]
    fn remember_is_idempotent_and_retrieve_uses_the_rag_index() {
        let user = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        let request = write_remember_request(user.path());
        let remember_arguments = vec![
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--request".to_owned(),
            request.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let inserted = run_remember(&remember_arguments).expect("remember insert");
        assert_eq!(inserted.status, "success");
        let claims = user.path().join(".hive/knowledge/Claims/user-root");
        assert_eq!(fs::read_dir(&claims).expect("claims").count(), 1);
        let repeated = run_remember(&remember_arguments).expect("remember no-op");
        assert!(repeated.changed_paths.is_empty());
        assert_eq!(fs::read_dir(&claims).expect("claims").count(), 1);

        let retrieve_arguments = vec![
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--target".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--scope".to_owned(),
            "global".to_owned(),
            "--query".to_owned(),
            "concise commit guidance".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let retrieved = run_retrieve(&retrieve_arguments).expect("retrieve");
        let hits = retrieved.data.expect("retrieval data")["hits"]
            .as_array()
            .expect("hits")
            .clone();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["collection_id"], "user-root");
        assert_eq!(hits[0]["untrusted_content"], true);
    }

    #[test]
    fn unregistered_target_auto_retrieves_user_root_and_shared_only() {
        let (user, _project) = registered_roots(true);
        let unregistered = temp_root();
        let store = RagStore::open(user.path()).expect("store");
        ensure_rag_registry(user.path(), &store).expect("normalized registry");
        let registry = store.load_registry().expect("collection registry");
        let CollectionResolution::Resolved(project_collection) =
            registry.resolve_project("project-test")
        else {
            panic!("project collection");
        };

        run_remember(&[
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--user-statement".to_owned(),
            "Installwide user beacon remains available without project setup.".to_owned(),
            "--claim-key".to_owned(),
            "installwide-user-beacon".to_owned(),
            "--kind".to_owned(),
            "workflow".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("user-root claim");
        for (file_name, fact, visibility) in [
            (
                "installwide-shared.json",
                "Installwide shared beacon is visible across projects.",
                RagVisibility::Shared,
            ),
            (
                "installwide-private.json",
                "Installwide private beacon stays inside its project.",
                RagVisibility::ProjectPrivate,
            ),
        ] {
            let request = write_collection_claim_request(
                user.path(),
                file_name,
                &project_collection,
                fact,
                visibility,
            );
            run_remember(&[
                "--user-root".to_owned(),
                user.path().to_string_lossy().into_owned(),
                "--request".to_owned(),
                request.to_string_lossy().into_owned(),
                "--output".to_owned(),
                "json".to_owned(),
            ])
            .expect("project claim");
        }

        let root_hits =
            auto_retrieval_hits(user.path(), unregistered.path(), "installwide user beacon");
        assert!(root_hits
            .iter()
            .any(|hit| hit["collection_id"] == "user-root"));
        let shared_hits = auto_retrieval_hits(
            user.path(),
            unregistered.path(),
            "installwide shared beacon",
        );
        assert!(shared_hits
            .iter()
            .any(|hit| hit["collection_id"] == project_collection));
        assert!(auto_retrieval_hits(
            user.path(),
            unregistered.path(),
            "installwide private beacon"
        )
        .iter()
        .all(|hit| !hit["text"]
            .as_str()
            .expect("hit text")
            .contains("private beacon")));

        let Err(error) = run_retrieve(&[
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--target".to_owned(),
            unregistered.path().to_string_lossy().into_owned(),
            "--query".to_owned(),
            "installwide private beacon".to_owned(),
            "--authorization-id".to_owned(),
            "unused".to_owned(),
            "--authorization-token".to_owned(),
            "unused".to_owned(),
            "--capabilities".to_owned(),
            "unused".to_owned(),
            "--usage".to_owned(),
            "unused".to_owned(),
        ]) else {
            panic!("unregistered confidential retrieval must fail closed");
        };
        assert_eq!(error.code(), "hive.knowledge-invalid-input");
    }

    #[test]
    fn user_statement_remember_needs_no_request_json_and_is_idempotent() {
        let user = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        let arguments = vec![
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--user-statement".to_owned(),
            "The user has a web background and is transitioning into game development.".to_owned(),
            "--claim-key".to_owned(),
            "career-background-web-to-game".to_owned(),
            "--kind".to_owned(),
            "project-profile".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let inserted = run_remember(&arguments).expect("user statement insert");
        assert_eq!(inserted.status, "success");
        let data = inserted.data.expect("insert data");
        assert_eq!(data["plan"]["new_claim"]["collection_id"], "user-root");
        assert_eq!(data["plan"]["new_claim"]["status"], "user-stated");
        assert_eq!(data["plan"]["new_claim"]["visibility"], "shared");
        assert!(data["plan"]["new_claim"]["locator"]
            .as_str()
            .expect("canonical locator")
            .starts_with(".hive/knowledge/Claims/user-root/claim-"));

        let repeated = run_remember(&arguments).expect("user statement no-op");
        assert!(repeated.changed_paths.is_empty());
    }

    #[test]
    fn user_statement_remember_accepts_safe_release_claim_key_and_blocks_credentials() {
        let user = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        let safe = run_remember(&[
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--user-statement".to_owned(),
            "Every Hive Skill description starts with its canonical English identifier.".to_owned(),
            "--claim-key".to_owned(),
            "v094-skill-description-and-projection-validation".to_owned(),
            "--kind".to_owned(),
            "convention".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("safe release claim");
        assert_eq!(safe.status, "success");
        RagStore::open(user.path())
            .expect("store")
            .validate_current()
            .expect("canonical Markdown and derived index");

        let blocked = temp_root();
        write_user_setup(blocked.path(), true);
        write_empty_knowledge(blocked.path());
        let Err(error) = run_remember(&[
            "--user-root".to_owned(),
            blocked.path().to_string_lossy().into_owned(),
            "--user-statement".to_owned(),
            "The access token is ghp_abcdefghijklmnopqrstuvwxyz012345.".to_owned(),
            "--claim-key".to_owned(),
            "release-credential".to_owned(),
            "--kind".to_owned(),
            "decision".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ]) else {
            panic!("credential must fail before user-root mutation");
        };
        assert_eq!(error.code(), "hive.knowledge-invalid-input");
        assert!(error
            .to_string()
            .contains("normalized_fact contains likely credential material"));
        assert!(!blocked
            .path()
            .join(".hive/knowledge/Claims/user-root")
            .exists());
        assert!(!blocked.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn user_statement_remember_rejects_unsupported_kind_and_mixed_request() {
        let user = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        let invalid_kind = vec![
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--user-statement".to_owned(),
            "A verified result.".to_owned(),
            "--claim-key".to_owned(),
            "result".to_owned(),
            "--kind".to_owned(),
            "outcome".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let Err(error) = run_remember(&invalid_kind) else {
            panic!("unsupported user-statement kind must fail");
        };
        assert_eq!(error.code(), "hive.knowledge-invalid-input");

        let request = write_remember_request(user.path());
        let mixed = vec![
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--request".to_owned(),
            request.to_string_lossy().into_owned(),
            "--user-statement".to_owned(),
            "A preference.".to_owned(),
            "--claim-key".to_owned(),
            "preference".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let Err(error) = run_remember(&mixed) else {
            panic!("mixed remember inputs must fail");
        };
        assert_eq!(error.code(), "hive.knowledge-invalid-input");
    }

    #[test]
    fn disabled_wiki_blocks_remember_before_store_initialization() {
        let user = temp_root();
        write_user_setup(user.path(), false);
        write_empty_knowledge(user.path());
        let request = write_remember_request(user.path());
        let arguments = vec![
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--request".to_owned(),
            request.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let Err(error) = run_remember(&arguments) else {
            panic!("disabled Wiki must reject remember");
        };
        assert_eq!(error.code(), "hive.knowledge-conflict");
        assert!(!user.path().join(".hive/config/collections.yml").exists());
        assert!(!user.path().join(SHARED_INDEX_RELATIVE).exists());
    }

    #[test]
    fn retrieve_requires_explicit_refresh_and_never_initializes_the_store() {
        let user = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        let arguments = vec![
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--target".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--scope".to_owned(),
            "global".to_owned(),
            "--query".to_owned(),
            "anything".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        assert!(matches!(
            run_retrieve(&arguments),
            Err(WikiError::Verification(_))
        ));
        for relative in [
            ".hive/config/collections.yml",
            ".hive/index/rag-generation.json",
            SHARED_INDEX_RELATIVE,
        ] {
            assert!(
                !user.path().join(relative).exists(),
                "retrieve created {relative}"
            );
        }
    }

    struct ConfidentialFixture {
        user: TempDir,
        project_a: TempDir,
        project_b: TempDir,
        project_b_collection: String,
        capability: PathBuf,
        usage: PathBuf,
    }

    fn confidential_fixture() -> ConfidentialFixture {
        use hive_wiki::rag::RagVisibility;

        let (user, project_b) = registered_roots(true);
        let project_a = temp_root();
        write_empty_knowledge(project_a.path());
        register_project(
            user.path(),
            RegisteredProject {
                id: "project-a".to_owned(),
                root: hive_wiki::shared::canonical_root(project_a.path())
                    .expect("canonical project A"),
                enabled: true,
                language: KnowledgeLanguage::En,
                visibility: KnowledgeVisibility::ProjectPrivate,
            },
        )
        .expect("register project A");
        let store = RagStore::open(user.path()).expect("store");
        ensure_rag_registry(user.path(), &store).expect("normalized registry");
        let registry = store.load_registry().expect("collection registry");
        let CollectionResolution::Resolved(project_b_collection) =
            registry.resolve_project("project-test")
        else {
            panic!("project B collection");
        };
        let request = write_collection_claim_request(
            user.path(),
            "confidential-request.json",
            &project_b_collection,
            "The sealed covenant uses cobalt release gates.",
            RagVisibility::Confidential,
        );
        run_remember(&[
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--request".to_owned(),
            request.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("confidential claim");

        let capability = user.path().join("capability.json");
        let usage = user.path().join("usage.json");
        fs::write(
            &capability,
            br#"{"schema_version":1,"resolved_owner":"host-native"}"#,
        )
        .expect("capability fixture");
        fs::write(&usage, br#"{"schema_version":1,"decision":"allowed"}"#).expect("usage fixture");

        ConfidentialFixture {
            user,
            project_a,
            project_b,
            project_b_collection,
            capability,
            usage,
        }
    }

    fn confidential_authorization(
        fixture: &ConfidentialFixture,
        target: &Path,
        scope: &str,
        nonce: &str,
    ) -> (String, String) {
        let issued = run_authorize_confidential(&[
            "--user-root".to_owned(),
            fixture.user.path().to_string_lossy().into_owned(),
            "--target".to_owned(),
            target.to_string_lossy().into_owned(),
            "--collection".to_owned(),
            "project-test".to_owned(),
            "--query".to_owned(),
            "sealed covenant cobalt".to_owned(),
            "--scope".to_owned(),
            scope.to_owned(),
            "--capabilities".to_owned(),
            fixture.capability.to_string_lossy().into_owned(),
            "--usage".to_owned(),
            fixture.usage.to_string_lossy().into_owned(),
            "--expires-at".to_owned(),
            (unix_now().expect("clock") + 30).to_string(),
            "--nonce".to_owned(),
            nonce.to_owned(),
            "--confirm-current-action".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("authorization")
        .data
        .expect("authorization data");
        (
            issued["authorization_id"]
                .as_str()
                .expect("authorization id")
                .to_owned(),
            issued["authorization_token"]
                .as_str()
                .expect("authorization token")
                .to_owned(),
        )
    }

    fn confidential_retrieve_arguments(
        fixture: &ConfidentialFixture,
        target: &Path,
        scope: &str,
        query: &str,
        authorization: Option<(&str, &str)>,
    ) -> Vec<String> {
        let mut arguments = vec![
            "--user-root".to_owned(),
            fixture.user.path().to_string_lossy().into_owned(),
            "--target".to_owned(),
            target.to_string_lossy().into_owned(),
            "--scope".to_owned(),
            scope.to_owned(),
            "--query".to_owned(),
            query.to_owned(),
        ];
        if let Some((authorization_id, token)) = authorization {
            arguments.extend([
                "--authorization-id".to_owned(),
                authorization_id.to_owned(),
                "--authorization-token".to_owned(),
                token.to_owned(),
                "--capabilities".to_owned(),
                fixture.capability.to_string_lossy().into_owned(),
                "--usage".to_owned(),
                fixture.usage.to_string_lossy().into_owned(),
            ]);
        }
        arguments.extend(["--output".to_owned(), "json".to_owned()]);
        arguments
    }

    #[test]
    fn current_project_confidential_requires_one_time_authorization() {
        let fixture = confidential_fixture();
        let without_authorization = run_retrieve(&confidential_retrieve_arguments(
            &fixture,
            fixture.project_b.path(),
            "auto",
            "sealed covenant cobalt",
            None,
        ))
        .expect("current project query remains bounded");
        assert!(without_authorization.data.expect("retrieval")["hits"]
            .as_array()
            .expect("hits")
            .is_empty());

        let authorization = confidential_authorization(
            &fixture,
            fixture.project_b.path(),
            "auto",
            "authorization-current-fixture-0001",
        );
        let arguments = confidential_retrieve_arguments(
            &fixture,
            fixture.project_b.path(),
            "auto",
            "sealed covenant cobalt",
            Some((&authorization.0, &authorization.1)),
        );
        let own_authorized = run_retrieve(&arguments).expect("current confidential token");
        assert_eq!(
            own_authorized.data.expect("own authorized retrieval")["hits"]
                .as_array()
                .expect("hits")
                .len(),
            1
        );
        assert!(run_retrieve(&arguments).is_err());
    }

    #[test]
    fn cross_project_confidential_authorization_rejects_forgery_and_replay() {
        let fixture = confidential_fixture();
        let without_authorization = run_retrieve(&confidential_retrieve_arguments(
            &fixture,
            fixture.project_a.path(),
            "project:project-test",
            "sealed covenant cobalt",
            None,
        ))
        .expect("unauthorized query remains bounded");
        assert!(without_authorization.data.expect("retrieval")["hits"]
            .as_array()
            .expect("hits")
            .is_empty());

        let authorization = confidential_authorization(
            &fixture,
            fixture.project_a.path(),
            "project:project-test",
            "authorization-fixture-0001",
        );
        let arguments = |target: &Path, query: &str, token: &str| {
            confidential_retrieve_arguments(
                &fixture,
                target,
                "project:project-test",
                query,
                Some((&authorization.0, token)),
            )
        };
        assert!(run_retrieve(&arguments(
            fixture.project_a.path(),
            "different query",
            &authorization.1,
        ))
        .is_err());
        assert!(run_retrieve(&arguments(
            fixture.project_b.path(),
            "sealed covenant cobalt",
            &authorization.1,
        ))
        .is_err());
        let mut forged = authorization.1.clone();
        forged.replace_range(0..1, if forged.starts_with('a') { "b" } else { "a" });
        assert!(run_retrieve(&arguments(
            fixture.project_a.path(),
            "sealed covenant cobalt",
            &forged,
        ))
        .is_err());

        let authorized = run_retrieve(&arguments(
            fixture.project_a.path(),
            "sealed covenant cobalt",
            &authorization.1,
        ))
        .expect("authorized cross-project confidential retrieval");
        assert_eq!(
            authorized.data.expect("authorized retrieval")["hits"][0]["collection_id"],
            fixture.project_b_collection
        );
        assert!(run_retrieve(&arguments(
            fixture.project_a.path(),
            "sealed covenant cobalt",
            &authorization.1,
        ))
        .is_err());

        assert!(run_retrieve(&[
            "--user-root".to_owned(),
            fixture.user.path().to_string_lossy().into_owned(),
            "--target".to_owned(),
            fixture.project_a.path().to_string_lossy().into_owned(),
            "--query".to_owned(),
            "sealed covenant cobalt".to_owned(),
            "--confidential-collection".to_owned(),
            fixture.project_b_collection,
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .is_err());
    }

    fn register_imported_collection(
        store: &RagStore,
        alias: &str,
        state: CollectionState,
        locator: Option<&Path>,
        portable_identity: &str,
    ) -> String {
        store
            .register_collection(CollectionRegistration {
                collection_id: None,
                kind: CollectionKind::Imported,
                state,
                aliases: vec![alias.to_owned()],
                local_locator: locator
                    .map(|root| hive_wiki::shared::canonical_root(root).expect("collection root")),
                source_project_id: None,
                default_visibility: CollectionVisibility::ProjectPrivate,
                portable_identity: Some(portable_identity.to_owned()),
                reviewed_inventory_digest: None,
            })
            .expect("imported collection")
            .collection
            .collection_id
    }

    fn collection_authorization(
        user_root: &Path,
        operation: &str,
        collection: &str,
        target: &Path,
        nonce: &str,
    ) -> (String, String) {
        let issued = run_authorize_collection(&[
            "--user-root".to_owned(),
            user_root.to_string_lossy().into_owned(),
            "--operation".to_owned(),
            operation.to_owned(),
            "--collection".to_owned(),
            collection.to_owned(),
            "--target".to_owned(),
            target.to_string_lossy().into_owned(),
            "--expires-at".to_owned(),
            (unix_now().expect("clock") + 30).to_string(),
            "--nonce".to_owned(),
            nonce.to_owned(),
            "--confirm-current-action".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("collection authorization")
        .data
        .expect("collection authorization data");
        (
            issued["authorization_id"]
                .as_str()
                .expect("authorization id")
                .to_owned(),
            issued["authorization_token"]
                .as_str()
                .expect("authorization token")
                .to_owned(),
        )
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn detached_collection_mapping_is_atomic_and_enables_private_auto_recall() {
        use hive_wiki::rag::RagVisibility;

        let user = temp_root();
        let original = temp_root();
        let mapped = temp_root();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        write_empty_knowledge(original.path());
        let store = RagStore::open(user.path()).expect("store");
        store.ensure_registry().expect("registry");
        let first = register_imported_collection(
            &store,
            "portable-project",
            CollectionState::Attached,
            Some(original.path()),
            "portable-project-one",
        );
        let request = write_collection_claim_request(
            user.path(),
            "portable-private-request.json",
            &first,
            "Portable private recall uses the aurora anchor.",
            RagVisibility::ProjectPrivate,
        );
        run_remember(&[
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--request".to_owned(),
            request.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("private claim");
        let (_, registry_digest) = store
            .load_registry_snapshot()
            .expect("collection registry snapshot");
        store
            .set_collection_attachment(&first, None, &registry_digest)
            .expect("detach imported collection");
        let mapping_authorization = collection_authorization(
            user.path(),
            "map",
            "portable-project",
            mapped.path(),
            "mapping-authorization-fixture-0001",
        );

        let attached = run_collection(&[
            "map".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--collection".to_owned(),
            "portable-project".to_owned(),
            "--target".to_owned(),
            mapped.path().to_string_lossy().into_owned(),
            "--authorization-id".to_owned(),
            mapping_authorization.0,
            "--authorization-token".to_owned(),
            mapping_authorization.1,
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("attach mapping");
        assert_eq!(attached.action, "MapKnowledgeCollection");
        let recalled = run_retrieve(&[
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--target".to_owned(),
            mapped.path().to_string_lossy().into_owned(),
            "--query".to_owned(),
            "aurora anchor".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .expect("mapped private recall");
        assert_eq!(
            recalled.data.expect("retrieval")["hits"][0]["collection_id"],
            first
        );

        let second = register_imported_collection(
            &store,
            "other-portable-project",
            CollectionState::Detached,
            None,
            "portable-project-two",
        );
        let conflict_authorization = collection_authorization(
            user.path(),
            "attach",
            &second,
            mapped.path(),
            "mapping-authorization-fixture-0002",
        );
        assert!(run_collection(&[
            "attach".to_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--collection".to_owned(),
            second.clone(),
            "--target".to_owned(),
            mapped.path().to_string_lossy().into_owned(),
            "--authorization-id".to_owned(),
            conflict_authorization.0,
            "--authorization-token".to_owned(),
            conflict_authorization.1,
            "--output".to_owned(),
            "json".to_owned(),
        ])
        .is_err());
        let registry = store.load_registry().expect("registry after conflict");
        let first_after = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == first)
            .expect("first collection remains");
        let second_after = registry
            .collections
            .iter()
            .find(|collection| collection.collection_id == second)
            .expect("second collection remains");
        assert_eq!(first_after.state, CollectionState::Attached);
        assert_eq!(second_after.state, CollectionState::Detached);
    }

    #[test]
    fn scan_apply_rejects_credentials_before_registry_or_index_mutation() {
        let user = temp_root();
        let target = temp_root_outside_repository();
        write_user_setup(user.path(), true);
        write_empty_knowledge(user.path());
        fs::write(target.path().join("README.md"), "# Safe project purpose\n")
            .expect("scan evidence");
        let scan = scan_directory(target.path(), false, None).expect("scan inventory fixture");
        let evidence = scan
            .inventory
            .entries
            .iter()
            .find(|entry| entry.relative_path == "README.md")
            .expect("README inventory row");
        let review = user.path().join("review.json");
        fs::write(
            &review,
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "inventory_digest": scan.inventory.inventory_digest,
                "claims": [{
                    "schema_version": 1,
                    "claim_id": "credential-claim",
                    "kind": "project-profile",
                    "statement": "Token sk-abcdefghijklmnopqrstuvwxyz0123456789",
                    "version": null,
                    "revision": null,
                    "applicability": null,
                    "evidence": [{
                        "locator": "README.md",
                        "content_digest": evidence.content_digest,
                        "kind": "document"
                    }],
                    "agent_reviewed": true,
                    "global_promotion_candidate": false
                }]
            }))
            .expect("review JSON"),
        )
        .expect("review file");
        let arguments = vec![
            "--target".to_owned(),
            target.path().to_string_lossy().into_owned(),
            "--apply".to_owned(),
            review.to_string_lossy().into_owned(),
            "--user-root".to_owned(),
            user.path().to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];

        let candidate_arguments = vec![
            "--target".to_owned(),
            target.path().to_string_lossy().into_owned(),
            "--candidates".to_owned(),
            review.to_string_lossy().into_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ];
        let Err(candidate_error) = run_scan(&candidate_arguments) else {
            panic!("candidate review must reject the credential before apply");
        };
        assert!(
            candidate_error
                .to_string()
                .contains("reviewed scan claim `credential-claim`"),
            "{candidate_error}"
        );
        let Err(apply_error) = run_scan(&arguments) else {
            panic!("apply must reject the credential");
        };
        assert_eq!(candidate_error.to_string(), apply_error.to_string());
        for relative in [
            ".hive/config/collections.yml",
            ".hive/index/rag-generation.json",
            SHARED_INDEX_RELATIVE,
            ".hive/index/rag-dirty.json",
            ".hive/knowledge/Claims",
        ] {
            assert!(
                !user.path().join(relative).exists(),
                "rejected scan created {relative}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn project_index_cleanup_rejects_symlinks_before_removing_regular_files() {
        use std::os::unix::fs::symlink;

        let project = temp_root();
        let external = temp_root();
        fs::create_dir_all(project.path().join(".hive/index")).expect("index directory");
        fs::write(project.path().join(SHARED_INDEX_RELATIVE), b"legacy")
            .expect("regular legacy index");
        let sentinel = external.path().join("sentinel");
        fs::write(&sentinel, b"outside").expect("external sentinel");
        symlink(
            &sentinel,
            project.path().join(".hive/index/hive.sqlite3-wal"),
        )
        .expect("unsafe sidecar");

        assert!(remove_legacy_project_indexes(project.path()).is_err());
        assert!(project.path().join(SHARED_INDEX_RELATIVE).is_file());
        assert_eq!(fs::read(sentinel).expect("external bytes"), b"outside");
    }
}
