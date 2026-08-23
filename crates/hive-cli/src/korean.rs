//! Deterministic Korean output inspection and verified language-pack lifecycle.

use super::{emit_action_result, update_discovery::fetch_https, ActionResult, Evidence};
use hive_core::korean::{
    embedded_manifest_bytes, embedded_rules_bytes, inspect, sanitize_text, verify, KoreanProfile,
};
use hive_core::{ensure_consumer_target, ensure_no_symlink_ancestors, sha256_digest};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MAX_TEXT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PACK_BYTES: u64 = 2 * 1024 * 1024;
const UPSTREAM_VERSION_URL: &str =
    "https://raw.githubusercontent.com/epoko77-ai/im-not-ai/main/plugin.json";
const USAGE: &str = "\
Inspect and verify Korean text without calling a model provider.

USAGE:
    hive korean inspect --profile response|release-note|documentation|technical|verbatim --input <file> --output json
    hive korean verify --profile response|release-note|documentation|technical|verbatim --before <file> --after <file> --output json
    hive korean sanitize --input <file> --output-file <file> --output json
    hive korean pack check --output json
    hive korean pack status --target <consumer> --output json
    hive korean pack preview --target <consumer> --candidate <pack-dir> --output json
    hive korean pack activate --target <consumer> --candidate <pack-dir> --consent-digest <sha256:...> --confirm-pack --output json
    hive korean pack rollback --target <consumer> --output json
";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackManifest {
    schema_version: u32,
    pack_id: String,
    pack_version: String,
    transform_version: u32,
    upstream: String,
    upstream_commit: String,
    upstream_tree_digest: String,
    upstream_file_count: usize,
    upstream_symlink_count: usize,
    license: String,
    upstream_license_digest: String,
    shipped_license_digest: String,
    rules_digest: String,
    source_inventory: Vec<Value>,
    host_versions: Value,
    runtime_agents: Vec<String>,
    retired_agents: Vec<String>,
    raw_install_allowed: bool,
    floating_ref_allowed: bool,
    automatic_update_allowed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackPointer {
    schema_version: u32,
    pack_id: String,
    pack_version: String,
    manifest_digest: String,
    rules_digest: String,
    relative: String,
    previous: Option<Box<PackPointerSnapshot>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackPointerSnapshot {
    pack_id: String,
    pack_version: String,
    manifest_digest: String,
    rules_digest: String,
    relative: String,
}

impl From<&PackPointer> for PackPointerSnapshot {
    fn from(value: &PackPointer) -> Self {
        Self {
            pack_id: value.pack_id.clone(),
            pack_version: value.pack_version.clone(),
            manifest_digest: value.manifest_digest.clone(),
            rules_digest: value.rules_digest.clone(),
            relative: value.relative.clone(),
        }
    }
}

pub(crate) fn run(arguments: &[String]) -> ExitCode {
    if arguments.is_empty() || arguments.iter().any(|argument| argument == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = match arguments.first().map(String::as_str) {
        Some("inspect") => run_inspect(&arguments[1..]),
        Some("verify") => run_verify(&arguments[1..]),
        Some("sanitize") => run_sanitize(&arguments[1..]),
        Some("pack") => run_pack(&arguments[1..]),
        Some(action) => Err(format!("unknown Korean action: {action}")),
        None => unreachable!("empty arguments returned above"),
    };
    emit_action_result(&result.unwrap_or_else(failure))
}

fn run_inspect(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(arguments, &["--profile", "--input", "--output"])?;
    require_json(&options)?;
    let profile = KoreanProfile::parse(required(&options, "--profile")?)?;
    let path = PathBuf::from(required(&options, "--input")?);
    let bytes = read_regular_bounded(&path, MAX_TEXT_BYTES, "Korean input")?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "Korean input must be UTF-8".to_owned())?;
    let result = inspect(profile, text)?;
    Ok(success(
        "InspectKorean",
        "hive.korean-inspection-complete",
        "Korean text inspection completed without rewriting",
        json!(result),
        vec![Evidence {
            kind: "input",
            locator: "korean-input".to_owned(),
            digest: sha256_digest(&bytes),
        }],
    ))
}

fn run_verify(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(arguments, &["--profile", "--before", "--after", "--output"])?;
    require_json(&options)?;
    let profile = KoreanProfile::parse(required(&options, "--profile")?)?;
    let before_path = PathBuf::from(required(&options, "--before")?);
    let after_path = PathBuf::from(required(&options, "--after")?);
    let before = read_regular_bounded(&before_path, MAX_TEXT_BYTES, "Korean source")?;
    let after = read_regular_bounded(&after_path, MAX_TEXT_BYTES, "Korean candidate")?;
    let source =
        std::str::from_utf8(&before).map_err(|_| "Korean source must be UTF-8".to_owned())?;
    let candidate =
        std::str::from_utf8(&after).map_err(|_| "Korean candidate must be UTF-8".to_owned())?;
    let result = verify(profile, source, candidate)?;
    let accepted = result.accepted;
    Ok(ActionResult {
        schema_version: 1,
        action: "VerifyKorean",
        status: if accepted {
            "success"
        } else {
            "verification-failed"
        },
        exit_code: if accepted { 0 } else { 5 },
        code: if accepted {
            "hive.korean-verification-passed"
        } else {
            "hive.korean-verification-failed"
        },
        message: if accepted {
            "Korean candidate preserved every deterministic invariant".to_owned()
        } else {
            "Korean candidate requires exact-source fallback".to_owned()
        },
        changed_paths: Vec::new(),
        evidence: vec![
            Evidence {
                kind: "source",
                locator: "korean-source".to_owned(),
                digest: sha256_digest(&before),
            },
            Evidence {
                kind: "candidate",
                locator: "korean-candidate".to_owned(),
                digest: sha256_digest(&after),
            },
        ],
        next_action: (!accepted)
            .then_some("use the exact source text or apply a smaller local rewrite".to_owned()),
        data: Some(json!(result)),
    })
}

fn run_sanitize(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(arguments, &["--input", "--output-file", "--output"])?;
    require_json(&options)?;
    let input = PathBuf::from(required(&options, "--input")?);
    let output = PathBuf::from(required(&options, "--output-file")?);
    if input == output {
        return Err("sanitize output must differ from its input".to_owned());
    }
    let bytes = read_regular_bounded(&input, MAX_TEXT_BYTES, "Korean input")?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "Korean input must be UTF-8".to_owned())?;
    let sanitized = sanitize_text(text);
    atomic_write(&output, sanitized.as_bytes())?;
    Ok(ActionResult {
        schema_version: 1,
        action: "SanitizeKorean",
        status: "success",
        exit_code: 0,
        code: "hive.korean-sanitized",
        message: "text hygiene removed only approved invisible controls".to_owned(),
        changed_paths: vec![output.display().to_string()],
        evidence: vec![Evidence {
            kind: "output",
            locator: output.display().to_string(),
            digest: sha256_digest(sanitized.as_bytes()),
        }],
        next_action: None,
        data: Some(json!({
            "input_chars": text.chars().count(),
            "output_chars": sanitized.chars().count(),
            "changed": text != sanitized,
            "watermark_claim": false,
        })),
    })
}

fn run_pack(arguments: &[String]) -> Result<ActionResult, String> {
    let Some(action) = arguments.first().map(String::as_str) else {
        return Err("Korean pack requires an action".to_owned());
    };
    match action {
        "check" => pack_check(&arguments[1..]),
        "status" => pack_status(&arguments[1..]),
        "preview" => pack_preview(&arguments[1..]),
        "activate" => pack_activate(&arguments[1..]),
        "rollback" => pack_rollback(&arguments[1..]),
        _ => Err(format!("unknown Korean pack action: {action}")),
    }
}

fn pack_check(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(arguments, &["--output"])?;
    require_json(&options)?;
    let current = embedded_manifest()?;
    let bytes = fetch_https(UPSTREAM_VERSION_URL, 64 * 1024)
        .map_err(|error| format!("Korean upstream version check failed: {error}"))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Korean upstream metadata is invalid: {error}"))?;
    let latest = value["version"]
        .as_str()
        .ok_or_else(|| "Korean upstream metadata has no version".to_owned())?;
    Ok(success(
        "CheckKoreanPack",
        "hive.korean-pack-check-complete",
        "Korean upstream version check completed without activation",
        json!({
            "current_version": current.pack_version,
            "latest_version": latest,
            "update_available": latest != current.pack_version,
            "activated": false,
            "source": UPSTREAM_VERSION_URL,
        }),
        vec![Evidence {
            kind: "upstream-metadata",
            locator: UPSTREAM_VERSION_URL.to_owned(),
            digest: sha256_digest(&bytes),
        }],
    ))
}

fn pack_status(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(arguments, &["--target", "--output"])?;
    require_json(&options)?;
    let target = consumer_target(required(&options, "--target")?)?;
    let pointer = load_pointer(&target)?;
    let embedded = embedded_manifest()?;
    let data = pointer.as_ref().map_or_else(
        || json!({"source": "embedded", "pack_id": embedded.pack_id, "pack_version": embedded.pack_version, "rules_digest": embedded.rules_digest}),
        |value| json!({"source": "activated", "pack_id": value.pack_id, "pack_version": value.pack_version, "rules_digest": value.rules_digest, "relative": value.relative}),
    );
    Ok(success(
        "StatusKoreanPack",
        "hive.korean-pack-status",
        "Korean language-pack status resolved",
        data,
        Vec::new(),
    ))
}

fn pack_preview(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(arguments, &["--target", "--candidate", "--output"])?;
    require_json(&options)?;
    let target = consumer_target(required(&options, "--target")?)?;
    let candidate = PathBuf::from(required(&options, "--candidate")?);
    let (manifest, manifest_bytes, rules_bytes) = validate_candidate(&candidate)?;
    let consent_digest = consent_digest(&target, &manifest_bytes, &rules_bytes)?;
    Ok(success(
        "PreviewKoreanPack",
        "hive.korean-pack-preview",
        "Korean language-pack preview completed without writes",
        json!({
            "pack_id": manifest.pack_id,
            "pack_version": manifest.pack_version,
            "manifest_digest": sha256_digest(&manifest_bytes),
            "rules_digest": sha256_digest(&rules_bytes),
            "consent_digest": consent_digest,
            "classifications": ["rules-data", "engine-code", "host-surface"],
            "activated": false,
        }),
        vec![Evidence {
            kind: "candidate-manifest",
            locator: "korean-pack-candidate".to_owned(),
            digest: sha256_digest(&manifest_bytes),
        }],
    ))
}

fn pack_activate(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(
        arguments,
        &[
            "--target",
            "--candidate",
            "--consent-digest",
            "--output",
            "--confirm-pack",
        ],
    )?;
    require_json(&options)?;
    if optional(&options, "--confirm-pack") != Some("true") {
        return Err("Korean pack activation requires --confirm-pack".to_owned());
    }
    let target = consumer_target(required(&options, "--target")?)?;
    let candidate = PathBuf::from(required(&options, "--candidate")?);
    let (manifest, manifest_bytes, rules_bytes) = validate_candidate(&candidate)?;
    let expected = consent_digest(&target, &manifest_bytes, &rules_bytes)?;
    if required(&options, "--consent-digest")? != expected {
        return Err("Korean pack consent digest does not match the exact preview".to_owned());
    }
    let manifest_digest = sha256_digest(&manifest_bytes);
    let suffix = &manifest_digest[7..19];
    let relative = format!(
        ".hive/language-packs/packs/{}-{suffix}",
        manifest.pack_version
    );
    ensure_no_symlink_ancestors(&target, Path::new(&relative))
        .map_err(|error| error.to_string())?;
    let destination = target.join(&relative);
    if !destination.exists() {
        let staging = target
            .join(".hive/language-packs/staging")
            .join(format!("{}-{suffix}", manifest.pack_version));
        if staging.exists() {
            return Err("Korean pack staging path already exists".to_owned());
        }
        fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
        fs::write(staging.join("manifest.json"), &manifest_bytes)
            .map_err(|error| error.to_string())?;
        fs::write(staging.join("rules.json"), &rules_bytes).map_err(|error| error.to_string())?;
        let license = read_regular_bounded(
            &candidate.join("UPSTREAM-LICENSE.txt"),
            MAX_PACK_BYTES,
            "Korean pack license",
        )?;
        fs::write(staging.join("UPSTREAM-LICENSE.txt"), license)
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(
            destination
                .parent()
                .ok_or_else(|| "invalid pack destination".to_owned())?,
        )
        .map_err(|error| error.to_string())?;
        fs::rename(&staging, &destination).map_err(|error| error.to_string())?;
    }
    let previous = load_pointer(&target)?
        .as_ref()
        .map(PackPointerSnapshot::from);
    let pointer = PackPointer {
        schema_version: 1,
        pack_id: manifest.pack_id,
        pack_version: manifest.pack_version,
        manifest_digest: manifest_digest.clone(),
        rules_digest: sha256_digest(&rules_bytes),
        relative: relative.clone(),
        previous: previous.map(Box::new),
    };
    let pointer_path = target.join(".hive/language-packs/current.json");
    atomic_json(&pointer_path, &pointer)?;
    Ok(ActionResult {
        schema_version: 1,
        action: "ActivateKoreanPack",
        status: "success",
        exit_code: 0,
        code: "hive.korean-pack-activated",
        message: "Korean language pack activated after exact preview consent".to_owned(),
        changed_paths: vec![relative, ".hive/language-packs/current.json".to_owned()],
        evidence: vec![Evidence {
            kind: "language-pack",
            locator: ".hive/language-packs/current.json".to_owned(),
            digest: sha256_digest(
                &serde_json_canonicalizer::to_vec(&pointer).map_err(|error| error.to_string())?,
            ),
        }],
        next_action: None,
        data: Some(
            json!({"pack_version": pointer.pack_version, "manifest_digest": manifest_digest, "activated": true}),
        ),
    })
}

fn pack_rollback(arguments: &[String]) -> Result<ActionResult, String> {
    let options = parse_options(arguments, &["--target", "--output"])?;
    require_json(&options)?;
    let target = consumer_target(required(&options, "--target")?)?;
    let current =
        load_pointer(&target)?.ok_or_else(|| "no activated Korean pack exists".to_owned())?;
    let previous = current
        .previous
        .clone()
        .ok_or_else(|| "no prior Korean pack exists for rollback".to_owned())?;
    let destination = target.join(&previous.relative);
    if !destination.join("manifest.json").is_file() || !destination.join("rules.json").is_file() {
        return Err("prior Korean pack is incomplete".to_owned());
    }
    let pointer = PackPointer {
        schema_version: 1,
        pack_id: previous.pack_id,
        pack_version: previous.pack_version,
        manifest_digest: previous.manifest_digest,
        rules_digest: previous.rules_digest,
        relative: previous.relative,
        previous: Some(Box::new(PackPointerSnapshot::from(&current))),
    };
    atomic_json(&target.join(".hive/language-packs/current.json"), &pointer)?;
    Ok(ActionResult {
        schema_version: 1,
        action: "RollbackKoreanPack",
        status: "success",
        exit_code: 0,
        code: "hive.korean-pack-rolled-back",
        message: "Korean language pack rolled back atomically".to_owned(),
        changed_paths: vec![".hive/language-packs/current.json".to_owned()],
        evidence: Vec::new(),
        next_action: None,
        data: Some(json!({"pack_version": pointer.pack_version, "rolled_back": true})),
    })
}

fn embedded_manifest() -> Result<PackManifest, String> {
    let manifest: PackManifest = serde_json::from_slice(embedded_manifest_bytes())
        .map_err(|error| format!("embedded Korean manifest is invalid: {error}"))?;
    validate_manifest(&manifest, embedded_rules_bytes(), embedded_manifest_bytes())?;
    Ok(manifest)
}

fn validate_candidate(candidate: &Path) -> Result<(PackManifest, Vec<u8>, Vec<u8>), String> {
    let metadata = fs::symlink_metadata(candidate).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Korean pack candidate must be a no-follow directory".to_owned());
    }
    let names = fs::read_dir(candidate)
        .map_err(|error| error.to_string())?
        .map(|entry| entry.map(|item| item.file_name().to_string_lossy().into_owned()))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()
        .map_err(|error| error.to_string())?;
    let allowed = ["manifest.json", "rules.json", "UPSTREAM-LICENSE.txt"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    if names != allowed {
        return Err("Korean pack candidate contains unknown or missing files".to_owned());
    }
    let manifest_bytes = read_regular_bounded(
        &candidate.join("manifest.json"),
        MAX_PACK_BYTES,
        "Korean pack manifest",
    )?;
    let rules_bytes = read_regular_bounded(
        &candidate.join("rules.json"),
        MAX_PACK_BYTES,
        "Korean pack rules",
    )?;
    let manifest: PackManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("Korean pack manifest is invalid: {error}"))?;
    validate_manifest(&manifest, &rules_bytes, &manifest_bytes)?;
    let license = read_regular_bounded(
        &candidate.join("UPSTREAM-LICENSE.txt"),
        MAX_PACK_BYTES,
        "Korean pack license",
    )?;
    if manifest.shipped_license_digest != sha256_digest(&license) {
        return Err("Korean pack license digest does not match its manifest".to_owned());
    }
    Ok((manifest, manifest_bytes, rules_bytes))
}

fn validate_manifest(
    manifest: &PackManifest,
    rules: &[u8],
    manifest_bytes: &[u8],
) -> Result<(), String> {
    if manifest.schema_version != 1
        || manifest.pack_id != "im-not-ai-korean-core"
        || manifest.transform_version == 0
        || manifest.license != "MIT"
        || manifest.upstream != "https://github.com/epoko77-ai/im-not-ai"
        || manifest.upstream_commit.len() != 40
        || !manifest
            .upstream_commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || manifest.upstream_symlink_count != 0
        || manifest.raw_install_allowed
        || manifest.floating_ref_allowed
        || manifest.automatic_update_allowed
        || manifest.rules_digest != sha256_digest(rules)
        || manifest.pack_version.is_empty()
        || manifest.source_inventory.is_empty()
        || manifest.runtime_agents.is_empty()
        || manifest.retired_agents.is_empty()
        || manifest.upstream_file_count == 0
        || !manifest.upstream_tree_digest.starts_with("sha256:")
        || !manifest.upstream_license_digest.starts_with("sha256:")
        || !manifest.shipped_license_digest.starts_with("sha256:")
        || manifest.host_versions.is_null()
        || u64::try_from(manifest_bytes.len()).unwrap_or(u64::MAX) > MAX_PACK_BYTES
    {
        return Err("Korean pack manifest violates the pinned provenance boundary".to_owned());
    }
    Ok(())
}

fn consent_digest(target: &Path, manifest: &[u8], rules: &[u8]) -> Result<String, String> {
    let target_digest = sha256_digest(
        target
            .to_str()
            .ok_or_else(|| "Korean pack target is not UTF-8".to_owned())?
            .as_bytes(),
    );
    let value = json!({
        "schema_version": 1,
        "action": "activate-korean-pack",
        "target_digest": target_digest,
        "manifest_digest": sha256_digest(manifest),
        "rules_digest": sha256_digest(rules),
    });
    Ok(sha256_digest(
        &serde_json_canonicalizer::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn load_pointer(target: &Path) -> Result<Option<PackPointer>, String> {
    let path = target.join(".hive/language-packs/current.json");
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            let bytes = read_regular_bounded(&path, MAX_PACK_BYTES, "Korean pack pointer")?;
            serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| format!("Korean pack pointer is invalid: {error}"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        _ => Err("Korean pack pointer is not a regular file".to_owned()),
    }
}

fn consumer_target(value: &str) -> Result<PathBuf, String> {
    let target = PathBuf::from(value);
    ensure_consumer_target(&target).map_err(|error| error.to_string())?;
    fs::canonicalize(target).map_err(|error| error.to_string())
}

fn read_regular_bounded(path: &Path, maximum: u64, label: &str) -> Result<Vec<u8>, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("cannot inspect {label}: {error}"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum {
        return Err(format!("{label} must be a bounded regular file"));
    }
    fs::read(path).map_err(|error| format!("cannot read {label}: {error}"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "output has no parent".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn atomic_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json_canonicalizer::to_vec(value).map_err(|error| error.to_string())?;
    atomic_write(path, &bytes)
}

type Options<'a> = Vec<(&'a str, &'a str)>;

fn parse_options<'a>(arguments: &'a [String], allowed: &[&str]) -> Result<Options<'a>, String> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        index += 1;
        if !allowed.contains(&option) || result.iter().any(|(existing, _)| *existing == option) {
            return Err(format!("unknown or duplicate Korean option: {option}"));
        }
        if option == "--confirm-pack" {
            result.push((option, "true"));
            continue;
        }
        let value = arguments
            .get(index)
            .ok_or_else(|| format!("missing value for {option}"))?;
        index += 1;
        result.push((option, value));
    }
    Ok(result)
}

fn required<'a>(options: &'a Options<'a>, name: &str) -> Result<&'a str, String> {
    optional(options, name).ok_or_else(|| format!("Korean command requires {name}"))
}

fn optional<'a>(options: &'a Options<'a>, name: &str) -> Option<&'a str> {
    options
        .iter()
        .find_map(|(option, value)| (*option == name).then_some(*value))
}

fn require_json(options: &Options<'_>) -> Result<(), String> {
    if optional(options, "--output") == Some("json") {
        Ok(())
    } else {
        Err("Korean command requires --output json".to_owned())
    }
}

fn success(
    action: &'static str,
    code: &'static str,
    message: &str,
    data: Value,
    evidence: Vec<Evidence>,
) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action,
        status: "success",
        exit_code: 0,
        code,
        message: message.to_owned(),
        changed_paths: Vec::new(),
        evidence,
        next_action: None,
        data: Some(data),
    }
}

fn failure(message: String) -> ActionResult {
    ActionResult {
        schema_version: 1,
        action: "KoreanLanguage",
        status: "error",
        exit_code: 2,
        code: "hive.korean-invalid-input",
        message,
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{consent_digest, embedded_manifest, validate_candidate};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn embedded_pack_is_pinned_and_raw_install_is_disabled() {
        let manifest = embedded_manifest().expect("embedded manifest");
        assert_eq!(manifest.pack_version, "2.3.2");
        assert_eq!(manifest.upstream_symlink_count, 0);
        assert!(!manifest.raw_install_allowed);
        assert!(!manifest.floating_ref_allowed);
        assert!(!manifest.automatic_update_allowed);
    }

    #[test]
    fn candidate_rejects_unknown_files_and_consent_binds_target() {
        let temp = TempDir::new().expect("temporary directory");
        let candidate = temp.path().join("candidate");
        fs::create_dir(&candidate).expect("candidate");
        fs::write(
            candidate.join("manifest.json"),
            hive_core::korean::embedded_manifest_bytes(),
        )
        .expect("manifest");
        fs::write(
            candidate.join("rules.json"),
            hive_core::korean::embedded_rules_bytes(),
        )
        .expect("rules");
        fs::write(
            candidate.join("UPSTREAM-LICENSE.txt"),
            hive_core::korean::embedded_license_bytes(),
        )
        .expect("license");
        let (_, manifest, rules) = validate_candidate(&candidate).expect("candidate");
        let first = consent_digest(temp.path(), &manifest, &rules).expect("first consent");
        let other = temp.path().join("other");
        fs::create_dir(&other).expect("other target");
        let second = consent_digest(&other, &manifest, &rules).expect("second consent");
        assert_ne!(first, second);
        fs::write(candidate.join("install.sh"), b"forbidden").expect("unknown file");
        assert!(validate_candidate(&candidate).is_err());
    }
}
