use crate::{SemVersion, UpdateError};
use hive_core::{is_hive_skill_projection_path, validate_project_relative};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const ALLOWED_MIGRATION_IDS: &[&str] = &[
    "same-major-render-v1",
    "cross-major-system-representation-v1",
];
const REQUIRED_PRESERVES: &[&str] = &[
    "project-files",
    "docs",
    "user-markdown-body",
    "foreign-bytes",
];
const REQUIRED_INPUTS: &[&str] = &["canonical-markdown", "typed-yaml", "typed-toml"];
const REQUIRED_FORBIDDEN_INPUTS: &[&str] = &["sqlite", "runtime", "backup"];

/// Migration route kind.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationKind {
    /// Backward-compatible render/update route.
    SameMajor,
    /// Explicitly approved breaking transform.
    CrossMajor,
}

/// A compiled Rust migration route declared by signed metadata.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationRoute {
    /// Stable route id.
    pub route_id: String,
    /// Inclusive minimum source product version.
    pub from_min: String,
    /// Inclusive maximum source product version.
    pub from_max: String,
    /// Exact target product version.
    pub to_version: String,
    /// Same-major or cross-major semantics.
    pub kind: MigrationKind,
    /// Compiled migration identifier. Metadata cannot provide executable code.
    pub migration_id: String,
    /// Canonical input classes.
    pub inputs: Vec<String>,
    /// Input classes that must never be read by migration.
    pub forbidden_inputs: Vec<String>,
    /// Byte/data classes that must be preserved.
    pub preserves: Vec<String>,
}

/// Signed migration route table.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationTable {
    /// Schema generation.
    pub schema_version: u32,
    /// Exact release target.
    pub target_version: String,
    /// Sorted, non-overlapping routes.
    pub routes: Vec<MigrationRoute>,
}

/// Digest evidence for one cross-major shadow-tree path.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreservationDigest {
    /// Project-relative path.
    pub path: String,
    /// Stable preservation class.
    pub kind: String,
    /// Exact SHA-256 digest.
    pub digest: String,
}

/// Validate that metadata selects only compiled, non-executable migration code.
///
/// # Errors
///
/// Returns an error for unsupported schemas, invalid or overlapping routes,
/// executable migration identifiers, or canonical-data preservation violations.
pub fn validate_migration_table(table: &MigrationTable) -> Result<(), UpdateError> {
    if table.schema_version != 1 {
        return Err(UpdateError::Input(
            "unsupported migration-table schema version".to_owned(),
        ));
    }
    let target: SemVersion = table
        .target_version
        .parse()
        .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
    if table.routes.is_empty() {
        return Err(UpdateError::Input(
            "migration table must contain at least one compiled route".to_owned(),
        ));
    }
    let mut route_ids = BTreeSet::new();
    let mut previous: Option<(SemVersion, SemVersion)> = None;
    for route in &table.routes {
        if route.route_id.is_empty() || !route_ids.insert(route.route_id.as_str()) {
            return Err(UpdateError::Input(
                "migration route ids must be non-empty and unique".to_owned(),
            ));
        }
        let minimum: SemVersion = route
            .from_min
            .parse()
            .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
        let maximum: SemVersion = route
            .from_max
            .parse()
            .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
        let route_target: SemVersion = route
            .to_version
            .parse()
            .map_err(|error: crate::ReleasePolicyError| UpdateError::Input(error.to_string()))?;
        if minimum > maximum || route_target != target {
            return Err(UpdateError::Input(
                "migration route range or target is invalid".to_owned(),
            ));
        }
        if !ALLOWED_MIGRATION_IDS.contains(&route.migration_id.as_str()) {
            return Err(UpdateError::Unsupported(format!(
                "migration id is not compiled into this release: {}",
                route.migration_id
            )));
        }
        if !contains_exact_set(&route.inputs, REQUIRED_INPUTS)
            || !contains_exact_set(&route.forbidden_inputs, REQUIRED_FORBIDDEN_INPUTS)
            || !contains_exact_set(&route.preserves, REQUIRED_PRESERVES)
        {
            return Err(UpdateError::Compatibility(
                "migration route violates canonical-data preservation".to_owned(),
            ));
        }
        match route.kind {
            MigrationKind::SameMajor
                if minimum.major != target.major
                    || maximum.major != target.major
                    || route.migration_id != "same-major-render-v1" =>
            {
                return Err(UpdateError::Compatibility(
                    "same-major route crosses a product major or selects the wrong implementation"
                        .to_owned(),
                ));
            }
            MigrationKind::CrossMajor
                if minimum.major == target.major
                    || maximum.major == target.major
                    || minimum.major != maximum.major
                    || route.migration_id != "cross-major-system-representation-v1" =>
            {
                return Err(UpdateError::Compatibility(
                    "cross-major route range or implementation is invalid".to_owned(),
                ));
            }
            MigrationKind::SameMajor | MigrationKind::CrossMajor => {}
        }
        if maximum >= target {
            return Err(UpdateError::Compatibility(
                "migration source range must precede the release target".to_owned(),
            ));
        }
        if let Some((_, previous_maximum)) = previous {
            if minimum <= previous_maximum {
                return Err(UpdateError::Input(
                    "migration route ranges overlap or are not sorted".to_owned(),
                ));
            }
        }
        previous = Some((minimum, maximum));
    }
    Ok(())
}

fn contains_exact_set(values: &[String], required: &[&str]) -> bool {
    let values: BTreeSet<&str> = values.iter().map(String::as_str).collect();
    let required: BTreeSet<&str> = required.iter().copied().collect();
    values == required
}

/// Select the single compiled route for an installed version.
///
/// # Errors
///
/// Returns an error when the table is invalid or exactly one compatible route
/// cannot be selected.
pub fn select_migration_route(
    table: &MigrationTable,
    installed: SemVersion,
) -> Result<&MigrationRoute, UpdateError> {
    validate_migration_table(table)?;
    let matching: Vec<&MigrationRoute> = table
        .routes
        .iter()
        .filter(|route| {
            let minimum = route.from_min.parse::<SemVersion>().ok();
            let maximum = route.from_max.parse::<SemVersion>().ok();
            minimum.is_some_and(|minimum| installed >= minimum)
                && maximum.is_some_and(|maximum| installed <= maximum)
        })
        .collect();
    match matching.as_slice() {
        [route] => Ok(*route),
        [] => Err(UpdateError::Unsupported(format!(
            "no migration route supports installed version {installed}"
        ))),
        _ => Err(UpdateError::Input(
            "multiple migration routes match the installed version".to_owned(),
        )),
    }
}

/// Prove that a cross-major shadow successor preserves user-owned byte classes.
///
/// Only explicitly named Hive system-representation paths may differ or be
/// added/removed. Project files, documents, preferences, and user-authored
/// Markdown bodies remain byte-identical.
///
/// # Errors
///
/// Returns an error for malformed evidence, derived-state inputs, unapproved
/// path changes, or any protected byte-class drift.
pub fn validate_cross_major_preservation(
    before: &[PreservationDigest],
    after: &[PreservationDigest],
    mutable_system_paths: &BTreeSet<String>,
) -> Result<(), UpdateError> {
    let before = validate_preservation_set(before)?;
    let after = validate_preservation_set(after)?;
    for path in mutable_system_paths {
        validate_project_relative(Path::new(path))
            .map_err(|error| UpdateError::Input(error.to_string()))?;
        if !cross_major_system_path(path) || !crate::backup_path_is_allowed(path) {
            return Err(UpdateError::Compatibility(
                "cross-major mutable paths must be exact Hive system representations".to_owned(),
            ));
        }
    }
    let all_paths: BTreeSet<&str> = before
        .keys()
        .chain(after.keys())
        .map(String::as_str)
        .collect();
    for path in all_paths {
        let prior = before.get(path);
        let successor = after.get(path);
        if mutable_system_paths.contains(path) {
            if prior
                .into_iter()
                .chain(successor)
                .any(|entry| protected_preservation_kind(&entry.kind))
            {
                return Err(UpdateError::Compatibility(
                    "protected user byte class cannot be a mutable system path".to_owned(),
                ));
            }
            continue;
        }
        if prior != successor {
            return Err(UpdateError::Compatibility(format!(
                "cross-major migration changed protected bytes: {path}"
            )));
        }
    }
    Ok(())
}

fn cross_major_system_path(path: &str) -> bool {
    (path.starts_with(".hive/config/")
        && !matches!(
            path,
            ".hive/config/role-seeds.yml"
                | ".hive/config/knowledge-scope.yml"
                | ".hive/config/approved-skills.yml"
                | ".hive/config/approved-hooks.yml"
        ))
        || matches!(
            path,
            ".hive/.gitignore" | ".hive/LICENSE-AIGENT-HIVE.txt" | ".hive/README.md"
        )
        || is_hive_skill_projection_path(Path::new(path))
}

fn validate_preservation_set(
    entries: &[PreservationDigest],
) -> Result<BTreeMap<String, PreservationDigest>, UpdateError> {
    let mut result = BTreeMap::new();
    for entry in entries {
        validate_project_relative(Path::new(&entry.path))
            .map_err(|error| UpdateError::Input(error.to_string()))?;
        if !crate::backup_path_is_allowed(&entry.path)
            || !is_sha256_digest(&entry.digest)
            || !matches!(
                entry.kind.as_str(),
                "project-file"
                    | "document"
                    | "preference"
                    | "user-markdown-body"
                    | "hive-system-representation"
            )
            || result.insert(entry.path.clone(), entry.clone()).is_some()
        {
            return Err(UpdateError::Input(
                "migration preservation evidence is invalid".to_owned(),
            ));
        }
    }
    Ok(result)
}

fn protected_preservation_kind(kind: &str) -> bool {
    matches!(
        kind,
        "project-file" | "document" | "preference" | "user-markdown-body"
    )
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(kind: MigrationKind) -> MigrationRoute {
        MigrationRoute {
            route_id: "route-1".to_owned(),
            from_min: if kind == MigrationKind::SameMajor {
                "0.1.0".to_owned()
            } else {
                "1.0.0".to_owned()
            },
            from_max: if kind == MigrationKind::SameMajor {
                "0.6.0".to_owned()
            } else {
                "1.9.9".to_owned()
            },
            to_version: if kind == MigrationKind::SameMajor {
                "0.7.0".to_owned()
            } else {
                "2.0.0".to_owned()
            },
            kind,
            migration_id: if kind == MigrationKind::SameMajor {
                "same-major-render-v1".to_owned()
            } else {
                "cross-major-system-representation-v1".to_owned()
            },
            inputs: vec![
                "canonical-markdown".to_owned(),
                "typed-yaml".to_owned(),
                "typed-toml".to_owned(),
            ],
            forbidden_inputs: REQUIRED_FORBIDDEN_INPUTS
                .iter()
                .map(ToString::to_string)
                .collect(),
            preserves: REQUIRED_PRESERVES.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn selects_the_single_compiled_same_major_route() {
        let table = MigrationTable {
            schema_version: 1,
            target_version: "0.7.0".to_owned(),
            routes: vec![route(MigrationKind::SameMajor)],
        };
        let selected =
            select_migration_route(&table, "0.6.0".parse().expect("valid installed version"))
                .expect("route");
        assert_eq!(selected.migration_id, "same-major-render-v1");
    }

    #[test]
    fn rejects_sqlite_input_and_arbitrary_executable_ids() {
        let mut sqlite = route(MigrationKind::SameMajor);
        sqlite.inputs.push("sqlite".to_owned());
        let table = MigrationTable {
            schema_version: 1,
            target_version: "0.7.0".to_owned(),
            routes: vec![sqlite],
        };
        assert!(matches!(
            validate_migration_table(&table),
            Err(UpdateError::Compatibility(_))
        ));

        let mut executable = route(MigrationKind::SameMajor);
        executable.migration_id = "run-script-from-release".to_owned();
        let table = MigrationTable {
            schema_version: 1,
            target_version: "0.7.0".to_owned(),
            routes: vec![executable],
        };
        assert!(matches!(
            validate_migration_table(&table),
            Err(UpdateError::Unsupported(_))
        ));

        let mut wrong_compiled_route = route(MigrationKind::SameMajor);
        wrong_compiled_route.migration_id = "cross-major-system-representation-v1".to_owned();
        let table = MigrationTable {
            schema_version: 1,
            target_version: "0.7.0".to_owned(),
            routes: vec![wrong_compiled_route],
        };
        assert!(matches!(
            validate_migration_table(&table),
            Err(UpdateError::Compatibility(_))
        ));
    }

    #[test]
    fn same_major_route_covers_every_supported_pre_release() {
        let table = MigrationTable {
            schema_version: 1,
            target_version: "0.7.0".to_owned(),
            routes: vec![route(MigrationKind::SameMajor)],
        };
        for installed in ["0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0", "0.6.0"] {
            assert_eq!(
                select_migration_route(&table, installed.parse().expect("version"))
                    .expect("route")
                    .migration_id,
                "same-major-render-v1"
            );
        }
    }

    #[test]
    fn cross_major_fixture_preserves_user_bytes_and_changes_only_system_representation() {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fixture {
            before: Vec<PreservationDigest>,
            after: Vec<PreservationDigest>,
            mutable_system_paths: BTreeSet<String>,
        }
        let fixture: Fixture = serde_json::from_str(include_str!(
            "../../../tests/fixtures/phase6/migrations/cross-major-preservation.json"
        ))
        .expect("fixture");
        validate_cross_major_preservation(
            &fixture.before,
            &fixture.after,
            &fixture.mutable_system_paths,
        )
        .expect("preservation");

        let mut tampered = fixture.after;
        tampered
            .iter_mut()
            .find(|entry| entry.kind == "document")
            .expect("document")
            .digest = format!("sha256:{}", "f".repeat(64));
        assert!(matches!(
            validate_cross_major_preservation(
                &fixture.before,
                &tampered,
                &fixture.mutable_system_paths,
            ),
            Err(UpdateError::Compatibility(_))
        ));
    }
}
