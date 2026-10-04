use super::{read_mergeable, valid_digest, BaseLedger, UpgradePlan, MAX_LEDGER_BYTES};
use cap_std::fs::Dir;
use hive_core::{is_hive_skill_projection_path, sha256_digest};
use hive_update::{three_way_merge, MergeDisposition, MergeOutcome, UpdateError};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

pub(super) fn input_result(target: &Dir, name: &str) -> Result<super::ProjectResult, UpdateError> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(UpdateError::Input(
            "invalid Skill merge input name".to_owned(),
        ));
    }
    let base = super::read_base_ledger(target, None)?.ok_or_else(|| {
        UpdateError::Verification(
            "Skill merge inputs require an authenticated project base".to_owned(),
        )
    })?;
    let candidate =
        hive_render::project_upgrade_candidate_in(target).map_err(super::render_error)?;
    if base.product_version == env!("CARGO_PKG_VERSION") {
        super::authenticate_current_base(&base, &candidate.files)?;
    }
    let mut files = Vec::new();
    for (path, incoming) in &candidate.files {
        if !is_hive_skill_projection_path(Path::new(path))
            || Path::new(path)
                .components()
                .nth(2)
                .is_none_or(|component| component.as_os_str() != name)
        {
            continue;
        }
        let local = read_mergeable(target, path)?;
        files.push(serde_json::json!({
            "path":path, "local_digest":local.as_deref().map(sha256_digest),
            "base_digest":base.files.iter().find(|file| file.path == *path).map(|file| &file.content_digest),
            "incoming_digest":sha256_digest(incoming),
            "incoming_content":std::str::from_utf8(incoming).map_err(|_| UpdateError::Verification("Skill merge source is not UTF-8".to_owned()))?,
        }));
    }
    if files.is_empty() {
        return Err(UpdateError::Input(
            "Skill merge input is not selected in this project".to_owned(),
        ));
    }
    Ok(super::ProjectResult {
        schema_version: 1,
        action: "UpdateProjectHarness",
        status: "success",
        exit_code: 0,
        code: "hive.project-skill-merge-inputs",
        message: "read-only Skill merge inputs prepared".to_owned(),
        changed_paths: Vec::new(),
        evidence: Vec::new(),
        next_action: None,
        data: Some(
            serde_json::json!({"schema_version":1,"product_version":env!("CARGO_PKG_VERSION"),
            "project_base_digest":base.ledger_digest,"skill_name":name,"files":files}),
        ),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SkillMergeRequest {
    schema_version: u32,
    product_version: String,
    project_base_digest: String,
    files: Vec<ReviewedSkillFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewedSkillFile {
    path: String,
    local_digest: String,
    incoming_digest: String,
    merged_content: String,
}

impl SkillMergeRequest {
    pub(super) fn load(path: &Path) -> Result<Self, UpdateError> {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|_| UpdateError::Input("skill merge request is unavailable".to_owned()))?;
        if !metadata.file_type().is_file() || metadata.len() > MAX_LEDGER_BYTES {
            return Err(UpdateError::Input(
                "skill merge request must be a bounded regular file".to_owned(),
            ));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|_| UpdateError::Input("skill merge request cannot be opened".to_owned()))?
            .take(MAX_LEDGER_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| UpdateError::Input("skill merge request cannot be read".to_owned()))?;
        if bytes.len() as u64 > MAX_LEDGER_BYTES {
            return Err(UpdateError::Input(
                "skill merge request exceeds its limit".to_owned(),
            ));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| UpdateError::Input("invalid skill merge request shape".to_owned()))
    }

    pub(super) fn validate(
        &self,
        target: &Dir,
        base: Option<&BaseLedger>,
        incoming: &BTreeMap<String, Vec<u8>>,
    ) -> Result<BTreeMap<String, MergeOutcome>, UpdateError> {
        let base = base.ok_or_else(|| {
            UpdateError::Verification(
                "reviewed Skill merging requires an authenticated project base".to_owned(),
            )
        })?;
        if self.schema_version != 1
            || self.product_version != env!("CARGO_PKG_VERSION")
            || self.project_base_digest != base.ledger_digest
            || self.files.is_empty()
            || self.files.len() > 64
        {
            return Err(UpdateError::Verification(
                "skill merge request does not match this project release".to_owned(),
            ));
        }
        let mut result = BTreeMap::new();
        for file in &self.files {
            super::validate_merge_path(Path::new(&file.path))?;
            if !is_hive_skill_projection_path(Path::new(&file.path))
                || !valid_digest(&file.local_digest)
                || !valid_digest(&file.incoming_digest)
                || file.merged_content.is_empty()
                || file.merged_content.len() > 1024 * 1024
                || result.contains_key(&file.path)
            {
                return Err(UpdateError::Input(
                    "invalid or duplicate reviewed Skill path".to_owned(),
                ));
            }
            let proposed = incoming.get(&file.path).ok_or_else(|| {
                UpdateError::Verification(
                    "reviewed Skill is not selected in the incoming release".to_owned(),
                )
            })?;
            let local = read_mergeable(target, &file.path)?.ok_or_else(|| {
                UpdateError::Conflict("reviewed Skill no longer exists".to_owned())
            })?;
            if sha256_digest(&local) != file.local_digest
                || sha256_digest(proposed) != file.incoming_digest
            {
                return Err(UpdateError::Conflict(
                    "reviewed Skill changed; review the new local and incoming contents".to_owned(),
                ));
            }
            let merged = file.merged_content.as_bytes();
            if file.path.ends_with("/SKILL.md") && skill_name(merged) != skill_name(proposed) {
                return Err(UpdateError::Verification(
                    "reviewed Skill must retain the incoming Skill name".to_owned(),
                ));
            }
            // Validate format against the current upstream reference, not an invented old base.
            // Authority comes solely from the explicit review and its approval digest.
            three_way_merge(
                Path::new(&file.path),
                Some(proposed),
                Some(merged),
                Some(proposed),
            )?;
            result.insert(
                file.path.clone(),
                MergeOutcome {
                    bytes: Some(merged.to_vec()),
                    disposition: MergeDisposition::Merged,
                    omitted_incoming_hunks: 0,
                    local_priority: true,
                },
            );
        }
        Ok(result)
    }
}

fn skill_name(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?.replace("\r\n", "\n");
    let header = text.strip_prefix("---\n")?.split_once("\n---\n")?.0;
    let metadata: serde_yaml::Value = serde_yaml::from_str(header).ok()?;
    metadata.get("name")?.as_str().map(str::to_owned)
}

pub(super) fn ensure_approved(
    expected: Option<&str>,
    supplied: Option<&str>,
) -> Result<(), UpdateError> {
    if expected.is_some_and(|digest| supplied != Some(digest)) {
        return Err(UpdateError::Input(
            "reviewed Skill apply requires the exact --approve-skill-merge digest from preview"
                .to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn ensure_no_unreviewed_overlaps(
    reports: &[super::PathReport],
) -> Result<(), UpdateError> {
    if reports.iter().any(|report| {
        is_hive_skill_projection_path(Path::new(&report.path))
            && report.incoming_digest.is_some()
            && report.omitted_incoming_hunks > 0
    }) {
        return Err(UpdateError::Conflict(
            "overlapping Skill changes require a reviewed combined Skill; incoming improvements would otherwise be omitted".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn approval_digest(target: &Path, plan: &UpgradePlan) -> Result<String, UpdateError> {
    let canonical_target = std::fs::canonicalize(target).map_err(|_| {
        UpdateError::Verification("skill merge target cannot be authenticated".to_owned())
    })?;
    let value = serde_json::json!({
        "schema_version": 1,
        "target_digest": sha256_digest(canonical_target.to_string_lossy().as_bytes()),
        "plan_digest": plan.plan_digest,
    });
    let bytes = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| UpdateError::Internal(error.to_string()))?;
    Ok(sha256_digest(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cap_std::ambient_authority;
    use serde_json::json;

    #[test]
    fn invalid_review_requests_preserve_local_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let path = ".agents/skills/ship/SKILL.md";
        std::fs::create_dir_all(root.join(".agents/skills/ship")).unwrap();
        let local = "---\nname: ship\ndescription: User rules\n---\n\nUser guard.\n";
        let new = "---\nname: ship\ndescription: Incoming rules\n---\n\nNew Hive guard.\n";
        std::fs::write(root.join(path), local).unwrap();
        let target = Dir::open_ambient_dir(root, ambient_authority()).unwrap();
        let base = BaseLedger {
            schema_version: 1,
            product_version: "0.11.0".to_owned(),
            files: Vec::new(),
            ledger_digest: sha256_digest(b"authenticated project base"),
        };
        let incoming = BTreeMap::from([(path.to_owned(), new.as_bytes().to_vec())]);
        let good = json!({"schema_version":1,"product_version":env!("CARGO_PKG_VERSION"),
            "project_base_digest":base.ledger_digest,"files":[{"path":path,
            "local_digest":sha256_digest(local.as_bytes()),"incoming_digest":sha256_digest(new.as_bytes()),
            "merged_content":format!("{new}\nUser guard.\n")}]});
        for fault in [
            "base",
            "version",
            "local",
            "incoming",
            "path",
            "duplicate",
            "empty",
            "rename",
        ] {
            let mut value = good.clone();
            match fault {
                "base" => value["project_base_digest"] = json!(sha256_digest(b"another base")),
                "version" => value["product_version"] = json!("0.11.0"),
                "local" => value["files"][0]["local_digest"] = json!(sha256_digest(b"stale local")),
                "incoming" => {
                    value["files"][0]["incoming_digest"] = json!(sha256_digest(b"stale incoming"));
                }
                "path" => value["files"][0]["path"] = json!(".hive/config/harness.toml"),
                "duplicate" => {
                    let entry = value["files"][0].clone();
                    value["files"].as_array_mut().unwrap().push(entry);
                }
                "empty" => value["files"][0]["merged_content"] = json!(""),
                "rename" => {
                    value["files"][0]["merged_content"] =
                        json!(new.replace("name: ship", "name: injected"));
                }
                _ => unreachable!(),
            }
            let request: SkillMergeRequest = serde_json::from_value(value).unwrap();
            assert!(
                request.validate(&target, Some(&base), &incoming).is_err(),
                "{fault}"
            );
            assert_eq!(std::fs::read(root.join(path)).unwrap(), local.as_bytes());
        }
        let request: SkillMergeRequest = serde_json::from_value(good).unwrap();
        assert!(request.validate(&target, None, &incoming).is_err());
    }

    #[test]
    fn unknown_fields_and_missing_files_are_not_merge_authority() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("request.json");
        std::fs::write(&path, b"{\"unrecognized\":\"do not echo this value\"}").unwrap();
        let error = SkillMergeRequest::load(&path).err().unwrap();
        assert!(!error.to_string().contains("do not echo"));
        assert!(SkillMergeRequest::load(temp.path()).is_err());
    }

    #[test]
    fn omitted_skill_improvements_require_review_but_retired_user_files_remain_preserved() {
        let mut report = super::super::PathReport {
            path: ".agents/skills/ship/SKILL.md".to_owned(),
            base_digest: Some(sha256_digest(b"old")),
            local_digest: Some(sha256_digest(b"custom")),
            incoming_digest: Some(sha256_digest(b"new")),
            final_digest: Some(sha256_digest(b"partial")),
            disposition: MergeDisposition::Merged,
            omitted_incoming_hunks: 1,
            local_priority: true,
        };
        assert!(ensure_no_unreviewed_overlaps(std::slice::from_ref(&report)).is_err());
        report.omitted_incoming_hunks = 0;
        ensure_no_unreviewed_overlaps(std::slice::from_ref(&report)).unwrap();
        report.omitted_incoming_hunks = 1;
        report.incoming_digest = None;
        ensure_no_unreviewed_overlaps(std::slice::from_ref(&report)).unwrap();
        report.incoming_digest = Some(sha256_digest(b"new directive"));
        report.path = ".agents/directives/00-project-harness.md".to_owned();
        ensure_no_unreviewed_overlaps(std::slice::from_ref(&report)).unwrap();
    }
}
