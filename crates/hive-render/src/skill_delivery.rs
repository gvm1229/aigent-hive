//! One physical delivery surface for plugin-covered built-in Skills.
use super::{io_internal, read_target_optional, RenderError, TargetRead};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use hive_core::sha256_digest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const PROVIDERS_PATH: &str = ".hive/config/skill-providers.json";
/// Verified plugin files keyed by the compiler's portable project-relative path.
pub type PluginSkillFiles = BTreeMap<String, Vec<u8>>;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkillProviders {
    pub schema_version: u32,
    pub product_version: String,
    pub provider: String,
    pub skills: BTreeMap<String, String>,
    pub local_skills: BTreeMap<String, String>,
}

fn instruction_bytes<'a>(path: &str, bytes: &'a [u8]) -> &'a [u8] {
    if path.ends_with("/SKILL.md") && bytes.starts_with(b"---\n") {
        if let Some(end) = bytes[4..].windows(5).position(|part| part == b"\n---\n") {
            return &bytes[4 + end + 5..];
        }
    }
    bytes
}

fn groups(files: &BTreeMap<PathBuf, Vec<u8>>) -> BTreeMap<String, BTreeMap<String, Vec<u8>>> {
    let mut result: BTreeMap<String, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
    for (path, bytes) in files {
        let path = path.to_string_lossy().replace('\\', "/");
        if let Some(rest) = path.strip_prefix(".agents/skills/") {
            if let Some((name, suffix)) = rest.split_once('/') {
                result
                    .entry(name.to_owned())
                    .or_default()
                    .insert(suffix.to_owned(), bytes.clone());
            }
        }
    }
    result
}

fn digest(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(match invocation_policy(files) {
        Some(true) => b"implicit\0",
        Some(false) => b"explicit\0",
        None => b"unverified\0",
    });
    for (path, content) in files {
        // Localized picker text is presentation; invocation policy is included above.
        if path == "agents/openai.yaml" {
            continue;
        }
        bytes.extend(path.as_bytes());
        bytes.push(0);
        bytes.extend(instruction_bytes(&format!("/{path}"), content));
        bytes.push(0);
    }
    sha256_digest(&bytes)
}

fn invocation_policy(files: &BTreeMap<String, Vec<u8>>) -> Option<bool> {
    let value: serde_yaml::Value = serde_yaml::from_slice(files.get("agents/openai.yaml")?).ok()?;
    value.get("policy")?.get("allow_implicit_invocation")?.as_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_body_requires_matching_verified_invocation_policy() {
        let temporary = tempfile::tempdir().unwrap();
        let target = crate::open_target_capability(temporary.path()).unwrap();
        let projection = hive_projection::compile_project_projection(
            hive_projection::Host::Codex, &["prompt-refine".to_owned()], &[],
        ).unwrap();
        let original = projection.files.into_iter()
            .map(|(path, bytes)| (PathBuf::from(path), bytes)).collect::<BTreeMap<_, _>>();
        for policy in [Some(true), Some(false), None] {
            let mut files = original.clone();
            let mut plugin = original.iter().map(|(path, bytes)| {
                (path.to_string_lossy().replace('\\', "/"), bytes.clone())
            }).collect::<PluginSkillFiles>();
            let metadata = ".agents/skills/prompt-refine/agents/openai.yaml";
            if policy == Some(false) {
                let bytes = plugin.get_mut(metadata).unwrap();
                *bytes = String::from_utf8(bytes.clone()).unwrap()
                    .replace("allow_implicit_invocation: true", "allow_implicit_invocation: false")
                    .into_bytes();
            } else if policy.is_none() {
                plugin.remove(metadata);
            }
            apply(&target, &mut files, &plugin).unwrap();
            let ledger: SkillProviders = serde_json::from_slice(&files[Path::new(PROVIDERS_PATH)]).unwrap();
            let local = files.contains_key(Path::new(".agents/skills/prompt-refine/SKILL.md"));
            assert_eq!(local, policy != Some(true));
            if local {
                assert_eq!(ledger.local_skills["prompt-refine"], "invocation-policy-different");
            } else {
                assert!(ledger.skills.contains_key("prompt-refine"));
            }
        }
    }
}

pub(crate) fn apply<T: TargetRead + ?Sized>(
    target: &T,
    files: &mut BTreeMap<PathBuf, Vec<u8>>,
    plugin: &PluginSkillFiles,
) -> Result<(), RenderError> {
    let plugin_files = plugin
        .iter()
        .map(|(path, bytes)| (PathBuf::from(path), bytes.clone()))
        .collect();
    let plugin_groups = groups(&plugin_files);
    let mut ledger = SkillProviders {
        schema_version: 1,
        product_version: env!("CARGO_PKG_VERSION").to_owned(),
        provider: "aigent-hive:codex".to_owned(),
        skills: BTreeMap::new(),
        local_skills: BTreeMap::new(),
    };
    for (name, expected) in groups(files) {
        let Some(provided) = plugin_groups.get(&name) else {
            ledger.local_skills.insert(
                name,
                if plugin.is_empty() {
                    "plugin-unavailable-or-unverified"
                } else {
                    "not-provided"
                }
                .to_owned(),
            );
            continue;
        };
        if invocation_policy(&expected).is_none()
            || invocation_policy(&expected) != invocation_policy(provided)
        {
            ledger.local_skills.insert(name, "invocation-policy-different".to_owned());
            continue;
        }
        if digest(&expected) != digest(provided) {
            ledger
                .local_skills
                .insert(name, "content-different".to_owned());
            continue;
        }
        let prefix = PathBuf::from(format!(".agents/skills/{name}"));
        let owned = expected
            .keys()
            .map(|suffix| prefix.join(suffix))
            .collect::<BTreeSet<_>>();
        if has_extra_files(&target.open_target_dir()?, &prefix, &owned)? {
            ledger
                .local_skills
                .insert(name, "unowned-resource".to_owned());
            continue;
        }
        let mut changed = false;
        let original = read_target_optional(target, Path::new(".hive/config/project-base.json"))?
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
        for (suffix, bytes) in &expected {
            let path = prefix.join(suffix);
            let prior = original
                .as_ref()
                .and_then(|value| value.get("files"))
                .and_then(serde_json::Value::as_array)
                .and_then(|files| {
                    files.iter().find(|entry| {
                        entry.get("path").and_then(serde_json::Value::as_str)
                            == Some(path.to_string_lossy().replace('\\', "/").as_str())
                    })
                })
                .and_then(|entry| entry.get("content"))
                .and_then(serde_json::Value::as_str);
            if read_target_optional(target, &path)?.is_some_and(|local| {
                local != *bytes && prior.map(str::as_bytes) != Some(local.as_slice())
            }) {
                changed = true;
                break;
            }
        }
        if changed {
            ledger
                .local_skills
                .insert(name, "modified-resource".to_owned());
            continue;
        }
        ledger.skills.insert(name, digest(&expected));
        for path in owned {
            files.remove(&path);
        }
    }
    files.insert(PathBuf::from(PROVIDERS_PATH), encode(&ledger)?);
    Ok(())
}

pub(crate) fn encode(ledger: &SkillProviders) -> Result<Vec<u8>, RenderError> {
    let mut bytes =
        serde_json::to_vec(ledger).map_err(|error| RenderError::Internal(error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Validate recorded provider content against an authenticated release tree.
///
/// # Errors
/// Returns an error for noncanonical records, version drift or altered content.
pub fn covered(
    bytes: Option<&[u8]>,
    expected: &BTreeMap<PathBuf, Vec<u8>>,
    version: &str,
) -> Result<BTreeSet<String>, RenderError> {
    let Some(bytes) = bytes else {
        return Ok(BTreeSet::new());
    };
    let ledger: SkillProviders = serde_json::from_slice(bytes).map_err(|error| {
        RenderError::Verification(format!("invalid Skill provider ledger: {error}"))
    })?;
    if ledger.schema_version != 1
        || ledger.product_version != version
        || ledger.provider != "aigent-hive:codex"
        || encode(&ledger)? != bytes
    {
        return Err(RenderError::Verification(
            "Skill provider ledger is not canonical or version-bound".to_owned(),
        ));
    }
    let expected = groups(expected);
    for (name, proof) in &ledger.skills {
        if expected
            .get(name)
            .is_none_or(|files| digest(files) != *proof)
        {
            return Err(RenderError::Verification(format!(
                "Skill provider content proof differs: {name}"
            )));
        }
    }
    for (name, reason) in &ledger.local_skills {
        if !expected.contains_key(name)
            || ledger.skills.contains_key(name)
            || !matches!(
                reason.as_str(),
                "plugin-unavailable-or-unverified"
                    | "not-provided"
                    | "content-different"
                    | "unowned-resource"
                    | "modified-resource"
            )
        {
            return Err(RenderError::Verification(format!(
                "invalid local Skill delivery record: {name}"
            )));
        }
    }
    if ledger.skills.len() + ledger.local_skills.len() != expected.len() {
        return Err(RenderError::Verification(
            "Skill provider ledger omits an active Skill".to_owned(),
        ));
    }
    Ok(ledger.skills.into_keys().collect())
}

/// Inspect names only. Unknown or nonregular entries preserve the whole Skill.
///
/// # Errors
/// Returns an error for an unsafe directory or an unreadable inventory.
pub fn has_extra_files(
    root: &Dir,
    path: &Path,
    owned: &BTreeSet<PathBuf>,
) -> Result<bool, RenderError> {
    let mut directory = root.try_clone().map_err(io_internal)?;
    for component in path.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err(RenderError::Safety(
                "Skill directory path is not normalized".to_owned(),
            ));
        }
        directory = match directory.open_dir_nofollow(component.as_os_str()) {
            Ok(directory) => directory,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(io_internal(error)),
        };
    }
    let entries = match directory.entries() {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(io_internal(error)),
    };
    for entry in entries {
        let entry = entry.map_err(io_internal)?;
        let child = path.join(entry.file_name());
        let kind = entry.file_type().map_err(io_internal)?;
        if kind.is_dir() {
            if has_extra_files(root, &child, owned)? {
                return Ok(true);
            }
        } else if !kind.is_file() || !owned.contains(&child) {
            return Ok(true);
        }
    }
    Ok(false)
}
