//! Data-only preparation for explicit optional-feature consent. No installation or model calls.
use super::{
    calculate_consent_digest, hook_descriptor_bytes, host_native_hook_event_supported,
    load_optional_skill_sources, parse_answers, sha256_digest, validate_hook_approvals,
    validate_skill_approvals, CapabilityResolution, HookApproval, RenderError, SkillApproval,
};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    io::Read,
    path::{Component, Path},
};

const HOOKS: &[(&str, &str)] = &[
    ("protect-hive-owned-state", "PreToolUse"),
    ("update-integrity-guard", "PreToolUse"),
    ("derived-state-invalidation", "PostToolUse"),
    ("checkpoint-reminder", "PreCompact"),
    ("checkpoint-reminder", "Stop"),
    ("continue-active-run", "Stop"),
    ("validate-korean-output", "Stop"),
    ("validate-korean-output", "AfterAgent"),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    #[serde(default)]
    skills: Vec<SkillRequest>,
    #[serde(default)]
    hooks: Vec<HookRequest>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillRequest {
    name: String,
    source: String,
    requested_capabilities: Vec<String>,
    approved_capabilities: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HookRequest {
    capability: String,
    event: String,
}

fn resolution(bytes: &[u8]) -> Result<CapabilityResolution, RenderError> {
    hive_core::run::CapabilityResolution::parse_json(bytes)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    serde_json::from_slice(bytes).map_err(|error| RenderError::Input(error.to_string()))
}

/// List only hook pairs supported by the supplied host evidence.
/// # Errors
/// Rejects invalid capability data.
pub fn supported_setup_hooks(capabilities: &[u8]) -> Result<Value, RenderError> {
    let resolution = resolution(capabilities)?;
    Ok(json!(HOOKS
        .iter()
        .filter(|(_, event)| host_native_hook_event_supported(&resolution, event))
        .map(|(capability, event)| json!({"capability":capability,"event":event}))
        .collect::<Vec<_>>()))
}

/// Extract editable choices without asking the user to reproduce hashes or timestamps.
/// # Errors
/// Rejects invalid saved setup data.
pub fn setup_extras_request(answers: &[u8]) -> Result<Value, RenderError> {
    let (answers, _) = parse_answers(answers)?;
    Ok(
        json!({"skills":answers.approved_optional_skills.iter().map(|skill|json!({"name":skill.name,"source":skill.source,
        "requested_capabilities":skill.requested_capabilities,"approved_capabilities":skill.approved_capabilities})).collect::<Vec<_>>(),
        "hooks":answers.approved_fallback_hooks.iter().map(|hook|json!({"capability":hook.capability,"event":hook.event})).collect::<Vec<_>>()}),
    )
}

/// Prepare complete approval records for a caller that obtains explicit confirmation.
/// Unchanged approvals retain their original time. This function never applies the records.
/// # Errors
/// Rejects unowned sources, invalid capability grants, unsupported hooks, or invalid dates.
pub fn materialize_setup_extras(
    target: &Path,
    request: &Value,
    previous: &[u8],
    capabilities: &[u8],
    answered_at: u64,
) -> Result<Value, RenderError> {
    super::ensure_consumer_target(target).map_err(|error| RenderError::Input(error.to_string()))?;
    let root = super::open_target_capability(target)?;
    let request: Request = serde_json::from_value(request.clone())
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let (previous, _) = parse_answers(previous)?;
    let resolution = resolution(capabilities)?;
    let approved_at = utc(answered_at)?;
    let mut skills = Vec::new();
    let mut names = BTreeSet::new();
    for input in request.skills {
        if !names.insert(input.name.clone()) {
            return Err(RenderError::Input("duplicate optional feature name".into()));
        }
        skills.push(skill_record(
            &root,
            input,
            &previous.approved_optional_skills,
            &approved_at,
        )?);
    }
    validate_skill_approvals(&skills)?;
    let sources = load_optional_skill_sources(&root, &skills)?;
    hive_projection::compile_projection(hive_projection::Host::Codex, &sources)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let mut hooks = Vec::new();
    let mut paths = BTreeSet::new();
    for input in request.hooks {
        if !HOOKS.contains(&(input.capability.as_str(), input.event.as_str()))
            || !paths.insert(input.capability.clone())
        {
            return Err(RenderError::Input(
                "choose one supported event per hook capability".into(),
            ));
        }
        let mut hook=HookApproval {consent_version:1,path:format!(".hive/hooks/{}",input.capability),
            command:format!("hive hook --capability {} --event {} --capabilities .hive/runtime/current-capability-resolution.json --output json",input.capability,input.event),
            capability:input.capability,event:input.event,content_digest:String::new(),approved_at:approved_at.clone(),consent_digest:String::new()};
        hook.content_digest = sha256_digest(&hook_descriptor_bytes(&hook)?);
        if let Some(old) = previous.approved_fallback_hooks.iter().find(|old| {
            old.capability == hook.capability
                && old.event == hook.event
                && old.content_digest == hook.content_digest
        }) {
            hook = old.clone();
        } else {
            hook.consent_digest = calculate_consent_digest(&hook)?;
        }
        hooks.push(hook);
    }
    validate_hook_approvals(&hooks, &resolution)?;
    Ok(json!({"approved_optional_skills":skills,"approved_fallback_hooks":hooks}))
}

fn skill_record(
    root: &cap_std::fs::Dir,
    mut input: SkillRequest,
    previous: &[SkillApproval],
    approved_at: &str,
) -> Result<SkillApproval, RenderError> {
    input.requested_capabilities.sort();
    input.approved_capabilities.sort();
    let old = previous.iter().find(|old| {
        old.name == input.name
            && old.source == input.source
            && old.requested_capabilities == input.requested_capabilities
            && old.approved_capabilities == input.approved_capabilities
    });
    let Some(relative) = input.source.strip_prefix("path:") else {
        return old.cloned().ok_or_else(|| {
            RenderError::Input(
                "new optional setup sources must use path:<project-relative-file>".into(),
            )
        });
    };
    let bytes = read_source(root, Path::new(relative))?;
    let digest = sha256_digest(&bytes);
    if let Some(old) = old.filter(|old| old.content_digest == digest) {
        return Ok(old.clone());
    }
    let mut skill = SkillApproval {
        consent_version: 1,
        name: input.name,
        source: input.source,
        revision: digest.clone(),
        content_digest: digest,
        requested_capabilities: input.requested_capabilities,
        approved_capabilities: input.approved_capabilities,
        approved_at: approved_at.into(),
        consent_digest: String::new(),
    };
    skill.consent_digest = calculate_consent_digest(&skill)?;
    Ok(skill)
}

fn read_source(root: &cap_std::fs::Dir, path: &Path) -> Result<Vec<u8>, RenderError> {
    hive_core::validate_project_relative(path)
        .map_err(|error| RenderError::Input(error.to_string()))?;
    let mut parent = root.try_clone().map_err(super::io_internal)?;
    for component in path.parent().into_iter().flat_map(Path::components) {
        let Component::Normal(name) = component else {
            return Err(RenderError::Input("invalid optional source path".into()));
        };
        parent = parent.open_dir_nofollow(name).map_err(super::io_internal)?;
    }
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = parent
        .open_with(
            path.file_name()
                .ok_or_else(|| RenderError::Input("missing optional source filename".into()))?,
            &options,
        )
        .map_err(super::io_internal)?;
    if !file.metadata().map_err(super::io_internal)?.is_file() {
        return Err(RenderError::Input(
            "optional source must be a regular file".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(super::io_internal)?;
    if bytes.len() > 1_048_576 {
        return Err(RenderError::Input(
            "optional source exceeds the size limit".into(),
        ));
    }
    Ok(bytes)
}

fn utc(seconds: u64) -> Result<String, RenderError> {
    let mut days = seconds / 86_400;
    let day_seconds = seconds % 86_400;
    let mut year = 1970_u64;
    let leap = |year: u64| {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    };
    loop {
        let length = if leap(year) { 366 } else { 365 };
        if days < length {
            break;
        }
        days -= length;
        year += 1;
        if year > 9999 {
            return Err(RenderError::Input(
                "consent time is outside the supported calendar".into(),
            ));
        }
    }
    let months = [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for length in months {
        if days < length {
            break;
        }
        days -= length;
        month += 1;
    }
    Ok(format!(
        "{year:04}-{month:02}-{:02}T{:02}:{:02}:{:02}Z",
        days + 1,
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/setup")
            .join(name)
    }

    #[test]
    fn consent_time_handles_epoch_leap_century_and_out_of_range_values() {
        for (seconds, expected) in [
            (0, "1970-01-01T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_709_251_199, "2024-02-29T23:59:59Z"),
            (4_107_542_400, "2100-03-01T00:00:00Z"),
        ] {
            assert_eq!(utc(seconds).unwrap(), expected);
        }
        assert!(utc(u64::MAX).is_err());
    }

    #[test]
    fn local_feature_and_supported_hook_records_bind_content_and_keep_unchanged_consent() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("vendor")).unwrap();
        let source=b"---\nname: local-check\ndescription: Read local fixture data.\n---\n\n# Local check\n";
        fs::write(root.path().join("vendor/SKILL.md"), source).unwrap();
        let previous = fs::read(fixture("answers-base.yml")).unwrap();
        let caps = fs::read(fixture("capabilities-codex-host-native-hooks.json")).unwrap();
        let request = json!({"skills":[{"name":"local-check","source":"path:vendor/SKILL.md","requested_capabilities":["filesystem-read"],"approved_capabilities":["filesystem-read"]}],
            "hooks":[{"capability":"checkpoint-reminder","event":"Stop"}]});
        let records =
            materialize_setup_extras(root.path(), &request, &previous, &caps, 1_709_251_199)
                .unwrap();
        assert_eq!(
            records["approved_optional_skills"][0]["content_digest"],
            sha256_digest(source)
        );
        assert_eq!(
            records["approved_fallback_hooks"][0]["approved_at"],
            "2024-02-29T23:59:59Z"
        );
        let mut answers: Value = serde_yaml::from_slice(&previous).unwrap();
        for key in ["approved_optional_skills", "approved_fallback_hooks"] {
            answers[key] = records[key].clone();
        }
        let saved = serde_json::to_vec(&answers).unwrap();
        assert_eq!(setup_extras_request(&saved).unwrap(), request);
        assert_eq!(
            materialize_setup_extras(root.path(), &request, &saved, &caps, 1_800_000_000).unwrap(),
            records
        );
        assert!(!root.path().join(".hive").exists());
        assert_eq!(
            fs::read(root.path().join("vendor/SKILL.md")).unwrap(),
            source
        );
    }

    #[test]
    fn unsupported_hooks_excess_grants_and_escaping_sources_never_become_approvals() {
        let root = tempfile::tempdir().unwrap();
        let previous = fs::read(fixture("answers-base.yml")).unwrap();
        let caps = fs::read(fixture("capabilities-codex-host-native.json")).unwrap();
        assert_eq!(supported_setup_hooks(&caps).unwrap(), json!([]));
        assert!(materialize_setup_extras(
            root.path(),
            &json!({"hooks":[{"capability":"checkpoint-reminder","event":"Stop"}]}),
            &previous,
            &caps,
            0
        )
        .is_err());
        for source in ["path:../foreign/SKILL.md", "path:/outside/SKILL.md"] {
            let request = json!({"skills":[{"name":"local-check","source":source,"requested_capabilities":[],"approved_capabilities":[]}]});
            assert!(materialize_setup_extras(root.path(), &request, &previous, &caps, 0).is_err());
        }
        fs::write(
            root.path().join("SKILL.md"),
            b"---\nname: local-check\ndescription: Read fixture.\n---\n",
        )
        .unwrap();
        let request = json!({"skills":[{"name":"local-check","source":"path:SKILL.md","requested_capabilities":["filesystem-read"],"approved_capabilities":["shell"]}]});
        assert!(materialize_setup_extras(root.path(), &request, &previous, &caps, 0).is_err());
        assert!(!root.path().join(".hive").exists());
    }

    #[test]
    fn unchanged_legacy_inert_approval_is_preserved_without_fetching_or_expanding_it() {
        let root = tempfile::tempdir().unwrap();
        let previous = fs::read(fixture("answers-approved-skill.yml")).unwrap();
        let caps = fs::read(fixture("capabilities-codex-host-native.json")).unwrap();
        let mut request = setup_extras_request(&previous).unwrap();
        let preserved =
            materialize_setup_extras(root.path(), &request, &previous, &caps, 0).unwrap();
        let original: Value = serde_yaml::from_slice(&previous).unwrap();
        assert_eq!(
            preserved["approved_optional_skills"],
            original["approved_optional_skills"]
        );
        request["skills"][0]["approved_capabilities"] =
            request["skills"][0]["requested_capabilities"].clone();
        assert!(materialize_setup_extras(root.path(), &request, &previous, &caps, 0).is_err());
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
