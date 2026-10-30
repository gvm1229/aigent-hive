//! Authenticate local-preservation receipts only for the projection surface being checked.
use super::{
    extract_exact_marker, is_hive_directive_projection_path, is_hive_skill_projection_path,
    is_shared_marker_path, read_target_optional, read_target_required, sha256_digest,
    shared_marker_foreign_digest_for_path, validate_hive_directive_projection_relative,
    validate_hive_skill_projection_relative, ProjectOverrideLedger, RenderError, TargetRead,
    PROJECT_OVERRIDES_PATH,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
pub(super) enum Scope {
    Skills,
    Directives,
    Shared,
    All,
}

pub(super) fn has_records<T: TargetRead + ?Sized>(
    target: &T,
    version: &str,
) -> Result<bool, RenderError> {
    Ok(read(target, version)?.is_some_and(|ledger| !ledger.files.is_empty()))
}

fn read<T: TargetRead + ?Sized>(
    target: &T,
    version: &str,
) -> Result<Option<ProjectOverrideLedger>, RenderError> {
    let Some(bytes) = read_target_optional(target, Path::new(PROJECT_OVERRIDES_PATH))? else {
        return Ok(None);
    };
    let ledger: ProjectOverrideLedger = serde_json::from_slice(&bytes).map_err(|error| {
        RenderError::Verification(format!("invalid project override ledger: {error}"))
    })?;
    if ledger.schema_version != 1 || ledger.product_version != version {
        return Err(RenderError::Verification("project override ledger does not match the installed Hive version; run hive project upgrade --apply".into()));
    }
    let mut unsigned =
        serde_json::to_value(&ledger).map_err(|error| RenderError::Internal(error.to_string()))?;
    unsigned
        .as_object_mut()
        .expect("ledger object")
        .remove("ledger_digest");
    let canonical = serde_json_canonicalizer::to_vec(&unsigned)
        .map_err(|error| RenderError::Internal(error.to_string()))?;
    if ledger.ledger_digest != sha256_digest(&canonical) {
        return Err(RenderError::Verification(
            "project override ledger digest is invalid; run hive project upgrade --apply".into(),
        ));
    }
    let mut canonical = serde_json_canonicalizer::to_vec(&ledger)
        .map_err(|error| RenderError::Internal(error.to_string()))?;
    canonical.push(b'\n');
    if bytes != canonical {
        return Err(RenderError::Verification(
            "project override ledger bytes are not canonical; run hive project upgrade --apply"
                .into(),
        ));
    }
    Ok(Some(ledger))
}

fn applies(path: &Path, scope: Scope) -> Result<bool, RenderError> {
    if is_hive_skill_projection_path(path) {
        validate_hive_skill_projection_relative(path)
            .map_err(|error| RenderError::Verification(error.to_string()))?;
        Ok(matches!(scope, Scope::Skills | Scope::All))
    } else if is_hive_directive_projection_path(path) {
        validate_hive_directive_projection_relative(path)
            .map_err(|error| RenderError::Verification(error.to_string()))?;
        Ok(matches!(scope, Scope::Directives | Scope::All))
    } else if is_shared_marker_path(path) {
        Ok(matches!(scope, Scope::Shared | Scope::All))
    } else {
        Err(RenderError::Verification(format!(
            "project override is outside the managed projection: {}",
            path.display()
        )))
    }
}

fn digest(path: &Path, bytes: &[u8]) -> Result<String, RenderError> {
    if is_shared_marker_path(path) {
        shared_marker_foreign_digest_for_path(path, bytes)?;
        Ok(sha256_digest(&extract_exact_marker(path, bytes)?))
    } else {
        Ok(sha256_digest(bytes))
    }
}

pub(super) fn paths<T: TargetRead + ?Sized>(
    target: &T,
    version: &str,
    expected: &BTreeMap<PathBuf, Vec<u8>>,
    scope: Scope,
) -> Result<BTreeSet<PathBuf>, RenderError> {
    let Some(ledger) = read(target, version)? else {
        return Ok(BTreeSet::new());
    };
    let mut seen = BTreeSet::new();
    let mut preserved = BTreeSet::new();
    for entry in ledger.files {
        let path = PathBuf::from(entry.path);
        if !seen.insert(path.clone()) {
            return Err(RenderError::Verification(
                "project override ledger contains duplicate paths".into(),
            ));
        }
        if !applies(&path, scope)? {
            continue;
        }
        let original = expected.get(&path).ok_or_else(|| {
            RenderError::Verification(format!(
                "project override path is not an active Hive projection: {}",
                path.display()
            ))
        })?;
        if digest(&path, original)? != entry.base_digest {
            return Err(RenderError::Verification(format!(
                "project override base digest is stale; run hive project upgrade --apply: {}",
                path.display()
            )));
        }
        let current = read_target_required(target, &path, "local-preserved projection")?;
        if digest(&path, &current)? != entry.local_digest {
            return Err(RenderError::Verification(format!(
                "local-preserved projection bytes changed after project upgrade: {}",
                path.display()
            )));
        }
        preserved.insert(path);
    }
    Ok(preserved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn save(target: &Path, entries: &[serde_json::Value]) {
        let mut ledger =
            json!({"schema_version":1,"product_version":env!("CARGO_PKG_VERSION"),"files":entries});
        ledger["ledger_digest"] = json!(sha256_digest(
            &serde_json_canonicalizer::to_vec(&ledger).unwrap()
        ));
        let mut bytes = serde_json_canonicalizer::to_vec(&ledger).unwrap();
        bytes.push(b'\n');
        let path = target.join(PROJECT_OVERRIDES_PATH);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn entry(path: &Path, original: &[u8], current: &[u8]) -> serde_json::Value {
        json!({"path":path.to_str().unwrap(),"base_digest":digest(path,original).unwrap(),
            "local_digest":digest(path,current).unwrap(),"omitted_incoming_hunks":0})
    }

    #[test]
    fn each_projection_scope_checks_its_own_fingerprints_and_all_checks_the_whole_ledger() {
        let root = tempfile::tempdir().unwrap();
        let mut expected = BTreeMap::new();
        let mut entries = Vec::new();
        for name in [
            "AGENTS.md",
            ".agents/directives/00-project-harness.md",
            ".agents/skills/ship/SKILL.md",
        ] {
            let path = Path::new(name);
            let original = if name == "AGENTS.md" {
                b"foreign\n\n<!-- AIGENT-HIVE:START -->\nbase\n<!-- AIGENT-HIVE:END -->\n".to_vec()
            } else {
                b"base".to_vec()
            };
            let current = if name == "AGENTS.md" {
                b"foreign\n\n<!-- AIGENT-HIVE:START -->\nlocal\n<!-- AIGENT-HIVE:END -->\n".to_vec()
            } else {
                b"local".to_vec()
            };
            let absolute = root.path().join(path);
            fs::create_dir_all(absolute.parent().unwrap()).unwrap();
            fs::write(absolute, &current).unwrap();
            entries.push(entry(path, &original, &current));
            expected.insert(path.to_owned(), original);
        }
        save(root.path(), &entries);
        assert_eq!(
            paths(
                root.path(),
                env!("CARGO_PKG_VERSION"),
                &expected,
                Scope::Skills
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            paths(
                root.path(),
                env!("CARGO_PKG_VERSION"),
                &expected,
                Scope::Directives
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            paths(
                root.path(),
                env!("CARGO_PKG_VERSION"),
                &expected,
                Scope::All
            )
            .unwrap()
            .len(),
            3
        );
        fs::write(
            root.path().join(".agents/directives/00-project-harness.md"),
            b"changed after receipt",
        )
        .unwrap();
        assert!(paths(
            root.path(),
            env!("CARGO_PKG_VERSION"),
            &expected,
            Scope::All
        )
        .is_err());
        let mut stale = entries.clone();
        stale[0]["base_digest"] = json!(sha256_digest(b"wrong base"));
        save(root.path(), &stale);
        assert!(paths(
            root.path(),
            env!("CARGO_PKG_VERSION"),
            &expected,
            Scope::All
        )
        .is_err());
        entries.push(entry(
            Path::new(".hive/knowledge/private.md"),
            b"base",
            b"local",
        ));
        save(root.path(), &entries);
        assert!(paths(
            root.path(),
            env!("CARGO_PKG_VERSION"),
            &expected,
            Scope::Skills
        )
        .is_err());
    }

    #[test]
    fn preserved_directives_and_markers_validate_but_ordinary_reconfiguration_never_overwrites_them(
    ) {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/setup");
        let answers = fixtures.join("answers-base.yml");
        let caps = fixtures.join("capabilities-codex-host-native.json");
        for name in [
            "AGENTS.md",
            ".agents/directives/00-project-harness.md",
            ".agents/skills/ship/SKILL.md",
        ] {
            let root = tempfile::tempdir().unwrap();
            let target = root.path().canonicalize().unwrap();
            let mut request = super::super::SetupRequest {
                target: &target,
                answers: &answers,
                capabilities: &caps,
                mode: super::super::SetupMode::Apply,
                reconfigure_roles: BTreeSet::new(),
                global_preferences: None,
            };
            super::super::execute_setup(&request).unwrap();
            let original = fs::read(target.join(name)).unwrap();
            let current = if name == "AGENTS.md" {
                String::from_utf8(original.clone())
                    .unwrap()
                    .replace(
                        "<!-- AIGENT-HIVE:END -->",
                        "- Local project rule.\n<!-- AIGENT-HIVE:END -->",
                    )
                    .into_bytes()
            } else {
                [original.as_slice(), b"\n- Local project rule.\n"].concat()
            };
            fs::write(target.join(name), &current).unwrap();
            save(&target, &[entry(Path::new(name), &original, &current)]);
            request.mode = super::super::SetupMode::Validate;
            super::super::execute_setup(&request).unwrap();
            for mode in [
                super::super::SetupMode::DryRun,
                super::super::SetupMode::Apply,
            ] {
                request.mode = mode;
                assert!(
                    matches!(super::super::execute_setup(&request),Err(RenderError::Conflict(message)) if message.contains("preserving reconfiguration"))
                );
                assert_eq!(fs::read(target.join(name)).unwrap(), current);
            }
        }
    }
}
