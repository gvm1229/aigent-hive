use crate::UpdateError;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const MAX_MERGE_BYTES: usize = 1024 * 1024;
const MAX_LCS_CELLS: usize = 2_000_000;

/// Stable three-way merge disposition.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MergeDisposition {
    /// Unmodified installed bytes replaced by the incoming release.
    IncomingReplace,
    /// User-local bytes retained because incoming did not change.
    LocalPreserved,
    /// Disjoint incoming hunks merged around local changes.
    Merged,
    /// New incoming path added.
    Added,
    /// Unmodified path removed by the incoming release.
    Deleted,
    /// Installed and incoming bytes already match.
    Unchanged,
}

/// Conflict-marker-free local-priority merge result.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct MergeOutcome {
    /// Final active bytes, or `None` for deletion.
    #[serde(skip)]
    pub bytes: Option<Vec<u8>>,
    /// Stable merge classification.
    pub disposition: MergeDisposition,
    /// Incoming hunks omitted because they overlap a local change.
    pub omitted_incoming_hunks: usize,
    /// Whether user-local precedence affected the result.
    pub local_priority: bool,
}

#[derive(Debug, Clone)]
struct Edit {
    start: usize,
    end: usize,
    replacement: Vec<String>,
    local: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TypedValue {
    Null,
    Boolean(bool),
    Integer(String),
    Float(String),
    String(String),
    DateTime(String),
    Sequence(Vec<Self>),
    Mapping(BTreeMap<String, Self>),
    Tagged(String, Box<Self>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TypedKind {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    DateTime,
    Sequence,
    Mapping,
    Tagged,
}

impl TypedValue {
    fn kind(&self) -> TypedKind {
        match self {
            Self::Null => TypedKind::Null,
            Self::Boolean(_) => TypedKind::Boolean,
            Self::Integer(_) => TypedKind::Integer,
            Self::Float(_) => TypedKind::Float,
            Self::String(_) => TypedKind::String,
            Self::DateTime(_) => TypedKind::DateTime,
            Self::Sequence(_) => TypedKind::Sequence,
            Self::Mapping(_) => TypedKind::Mapping,
            Self::Tagged(_, _) => TypedKind::Tagged,
        }
    }
}

/// Merge one authenticated base, live local value, and incoming release value.
///
/// Text merges add disjoint incoming hunks and omit overlapping incoming hunks
/// without writing conflict markers. JSON, YAML, and TOML inputs are parsed
/// before and after the line-preserving merge so unknown local fields and their
/// ordering remain byte-stable whenever their lines do not overlap.
///
/// # Errors
///
/// Returns a conflict when the authenticated base is missing for an occupied
/// path, input is non-UTF-8/oversized, typed syntax is invalid, or the bounded
/// diff cannot be computed safely.
pub fn three_way_merge(
    path: &Path,
    base: Option<&[u8]>,
    local: Option<&[u8]>,
    incoming: Option<&[u8]>,
) -> Result<MergeOutcome, UpdateError> {
    let Some(base) = base else {
        return match (local, incoming) {
            (None, Some(incoming)) => Ok(MergeOutcome {
                bytes: Some(incoming.to_vec()),
                disposition: MergeDisposition::Added,
                omitted_incoming_hunks: 0,
                local_priority: false,
            }),
            (None, None) => Ok(MergeOutcome {
                bytes: None,
                disposition: MergeDisposition::Unchanged,
                omitted_incoming_hunks: 0,
                local_priority: false,
            }),
            (Some(local), Some(incoming)) if local == incoming => Ok(MergeOutcome {
                bytes: Some(local.to_vec()),
                disposition: MergeDisposition::Unchanged,
                omitted_incoming_hunks: 0,
                local_priority: false,
            }),
            (Some(_), Some(_) | None) => Err(UpdateError::Conflict(format!(
                "occupied project projection lacks an authenticated three-way base: {}",
                path.display()
            ))),
        };
    };

    validate_merge_inputs(path, base, local, incoming)?;
    if let Some(outcome) = resolve_trivial_merge(base, local, incoming) {
        return Ok(outcome);
    }
    let Some(local) = local else {
        return Ok(MergeOutcome {
            bytes: None,
            disposition: MergeDisposition::LocalPreserved,
            omitted_incoming_hunks: usize::from(incoming.is_some()),
            local_priority: true,
        });
    };
    let Some(incoming) = incoming else {
        return Ok(MergeOutcome {
            bytes: Some(local.to_vec()),
            disposition: MergeDisposition::LocalPreserved,
            omitted_incoming_hunks: 1,
            local_priority: true,
        });
    };
    merge_changed_text(path, base, local, incoming)
}

fn validate_merge_inputs(
    path: &Path,
    base: &[u8],
    local: Option<&[u8]>,
    incoming: Option<&[u8]>,
) -> Result<(), UpdateError> {
    for bytes in [Some(base), local, incoming].into_iter().flatten() {
        if bytes.len() > MAX_MERGE_BYTES {
            return Err(UpdateError::Conflict(format!(
                "project projection exceeds the bounded merge limit: {}",
                path.display()
            )));
        }
        std::str::from_utf8(bytes).map_err(|_| {
            UpdateError::Conflict(format!(
                "project projection is not UTF-8: {}",
                path.display()
            ))
        })?;
        validate_typed(path, bytes)?;
    }
    Ok(())
}

fn resolve_trivial_merge(
    base: &[u8],
    local: Option<&[u8]>,
    incoming: Option<&[u8]>,
) -> Option<MergeOutcome> {
    if local == Some(base) {
        return Some(match incoming {
            Some(incoming) if incoming == base => MergeOutcome {
                bytes: Some(base.to_vec()),
                disposition: MergeDisposition::Unchanged,
                omitted_incoming_hunks: 0,
                local_priority: false,
            },
            Some(incoming) => MergeOutcome {
                bytes: Some(incoming.to_vec()),
                disposition: MergeDisposition::IncomingReplace,
                omitted_incoming_hunks: 0,
                local_priority: false,
            },
            None => MergeOutcome {
                bytes: None,
                disposition: MergeDisposition::Deleted,
                omitted_incoming_hunks: 0,
                local_priority: false,
            },
        });
    }
    if incoming == Some(base) || local == incoming {
        return Some(MergeOutcome {
            bytes: local.map(<[u8]>::to_vec),
            disposition: if local == incoming {
                MergeDisposition::Unchanged
            } else {
                MergeDisposition::LocalPreserved
            },
            omitted_incoming_hunks: 0,
            local_priority: incoming != local,
        });
    }
    None
}

fn merge_changed_text(
    path: &Path,
    base: &[u8],
    local: &[u8],
    incoming: &[u8],
) -> Result<MergeOutcome, UpdateError> {
    let utf8_error = |_| {
        UpdateError::Conflict(format!(
            "project projection is not UTF-8: {}",
            path.display()
        ))
    };
    let base_text = std::str::from_utf8(base).map_err(utf8_error)?;
    let local_text = std::str::from_utf8(local).map_err(utf8_error)?;
    let incoming_text = std::str::from_utf8(incoming).map_err(utf8_error)?;
    validate_typed_compatibility(path, base, local, incoming)?;
    let base_lines = lines(base_text);
    let local_lines = lines(local_text);
    let incoming_lines = lines(incoming_text);
    let mut local_edits = diff_edits(&base_lines, &local_lines, true, path)?;
    let incoming_edits = diff_edits(&base_lines, &incoming_lines, false, path)?;
    let mut omitted = 0;
    for incoming_edit in incoming_edits {
        if local_edits
            .iter()
            .any(|local_edit| edits_overlap(local_edit, &incoming_edit))
        {
            omitted += 1;
        } else {
            local_edits.push(incoming_edit);
        }
    }
    local_edits.sort_by_key(|edit| (edit.start, edit.end, !edit.local));
    let mut merged = String::new();
    let mut cursor = 0;
    for edit in local_edits {
        for line in &base_lines[cursor..edit.start] {
            merged.push_str(line);
        }
        for line in edit.replacement {
            merged.push_str(&line);
        }
        cursor = edit.end;
    }
    for line in &base_lines[cursor..] {
        merged.push_str(line);
    }
    validate_typed_merge_result(path, base, local, incoming, merged.as_bytes())?;
    Ok(MergeOutcome {
        bytes: Some(merged.into_bytes()),
        disposition: MergeDisposition::Merged,
        omitted_incoming_hunks: omitted,
        local_priority: true,
    })
}

fn lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split_inclusive('\n').map(ToOwned::to_owned).collect()
    }
}

fn diff_edits(
    base: &[String],
    variant: &[String],
    local: bool,
    path: &Path,
) -> Result<Vec<Edit>, UpdateError> {
    let rows = base.len().saturating_add(1);
    let columns = variant.len().saturating_add(1);
    let cells = rows.checked_mul(columns).ok_or_else(|| {
        UpdateError::Conflict(format!("merge diff size overflow: {}", path.display()))
    })?;
    if cells > MAX_LCS_CELLS {
        return Err(UpdateError::Conflict(format!(
            "project projection line diff exceeds the bounded merge limit: {}",
            path.display()
        )));
    }
    let mut lcs = vec![0_usize; cells];
    let index = |row: usize, column: usize| row * columns + column;
    for row in (0..base.len()).rev() {
        for column in (0..variant.len()).rev() {
            lcs[index(row, column)] = if base[row] == variant[column] {
                lcs[index(row + 1, column + 1)] + 1
            } else {
                lcs[index(row + 1, column)].max(lcs[index(row, column + 1)])
            };
        }
    }
    let mut edits = Vec::new();
    let mut row = 0;
    let mut column = 0;
    while row < base.len() || column < variant.len() {
        if row < base.len() && column < variant.len() && base[row] == variant[column] {
            row += 1;
            column += 1;
            continue;
        }
        let start = row;
        let mut replacement = Vec::new();
        while row < base.len() || column < variant.len() {
            if row < base.len() && column < variant.len() && base[row] == variant[column] {
                break;
            }
            if column < variant.len()
                && (row == base.len() || lcs[index(row, column + 1)] >= lcs[index(row + 1, column)])
            {
                replacement.push(variant[column].clone());
                column += 1;
            } else if row < base.len() {
                row += 1;
            }
        }
        edits.push(Edit {
            start,
            end: row,
            replacement,
            local,
        });
    }
    Ok(edits)
}

fn edits_overlap(left: &Edit, right: &Edit) -> bool {
    match (left.start == left.end, right.start == right.end) {
        (true, true) => left.start == right.start,
        (true, false) => (right.start..right.end).contains(&left.start),
        (false, true) => (left.start..left.end).contains(&right.start),
        (false, false) => left.start < right.end && right.start < left.end,
    }
}

fn validate_typed(path: &Path, bytes: &[u8]) -> Result<(), UpdateError> {
    parse_typed(path, bytes).map(|_| ())
}

fn parse_typed(path: &Path, bytes: &[u8]) -> Result<Option<TypedValue>, UpdateError> {
    let extension = path.extension().and_then(|value| value.to_str());
    let text = std::str::from_utf8(bytes)
        .map_err(|_| UpdateError::Conflict("typed merge input is not UTF-8".to_owned()))?;
    match extension {
        Some("json") => {
            let value = serde_json::from_str::<serde_json::Value>(text).map_err(|error| {
                UpdateError::Conflict(format!(
                    "JSON project projection is invalid at {}: {error}",
                    path.display()
                ))
            })?;
            Ok(Some(json_typed_value(value)))
        }
        Some("yaml" | "yml") => {
            let value = serde_yaml::from_str::<serde_yaml::Value>(text).map_err(|error| {
                UpdateError::Conflict(format!(
                    "YAML project projection is invalid at {}: {error}",
                    path.display()
                ))
            })?;
            Ok(Some(yaml_typed_value(path, value)?))
        }
        Some("toml") => {
            let value = toml::from_str::<toml::Value>(text).map_err(|error| {
                UpdateError::Conflict(format!(
                    "TOML project projection is invalid at {}: {error}",
                    path.display()
                ))
            })?;
            Ok(Some(toml_typed_value(value)))
        }
        _ => Ok(None),
    }
}

fn json_typed_value(value: serde_json::Value) -> TypedValue {
    match value {
        serde_json::Value::Null => TypedValue::Null,
        serde_json::Value::Bool(value) => TypedValue::Boolean(value),
        serde_json::Value::Number(value) if value.is_i64() || value.is_u64() => {
            TypedValue::Integer(value.to_string())
        }
        serde_json::Value::Number(value) => TypedValue::Float(value.to_string()),
        serde_json::Value::String(value) => TypedValue::String(value),
        serde_json::Value::Array(values) => {
            TypedValue::Sequence(values.into_iter().map(json_typed_value).collect())
        }
        serde_json::Value::Object(values) => TypedValue::Mapping(
            values
                .into_iter()
                .map(|(key, value)| (key, json_typed_value(value)))
                .collect(),
        ),
    }
}

fn yaml_typed_value(path: &Path, value: serde_yaml::Value) -> Result<TypedValue, UpdateError> {
    match value {
        serde_yaml::Value::Null => Ok(TypedValue::Null),
        serde_yaml::Value::Bool(value) => Ok(TypedValue::Boolean(value)),
        serde_yaml::Value::Number(value) if value.is_i64() || value.is_u64() => {
            Ok(TypedValue::Integer(value.to_string()))
        }
        serde_yaml::Value::Number(value) => Ok(TypedValue::Float(value.to_string())),
        serde_yaml::Value::String(value) => Ok(TypedValue::String(value)),
        serde_yaml::Value::Sequence(values) => values
            .into_iter()
            .map(|value| yaml_typed_value(path, value))
            .collect::<Result<Vec<_>, _>>()
            .map(TypedValue::Sequence),
        serde_yaml::Value::Mapping(values) => {
            let mut mapping = BTreeMap::new();
            for (key, value) in values {
                let serde_yaml::Value::String(key) = key else {
                    return Err(UpdateError::Conflict(format!(
                        "YAML project projection has a non-string mapping key at {}",
                        path.display()
                    )));
                };
                mapping.insert(key, yaml_typed_value(path, value)?);
            }
            Ok(TypedValue::Mapping(mapping))
        }
        serde_yaml::Value::Tagged(tagged) => Ok(TypedValue::Tagged(
            tagged.tag.to_string(),
            Box::new(yaml_typed_value(path, tagged.value)?),
        )),
    }
}

fn toml_typed_value(value: toml::Value) -> TypedValue {
    match value {
        toml::Value::String(value) => TypedValue::String(value),
        toml::Value::Integer(value) => TypedValue::Integer(value.to_string()),
        toml::Value::Float(value) => TypedValue::Float(value.to_string()),
        toml::Value::Boolean(value) => TypedValue::Boolean(value),
        toml::Value::Datetime(value) => TypedValue::DateTime(value.to_string()),
        toml::Value::Array(values) => {
            TypedValue::Sequence(values.into_iter().map(toml_typed_value).collect())
        }
        toml::Value::Table(values) => TypedValue::Mapping(
            values
                .into_iter()
                .map(|(key, value)| (key, toml_typed_value(value)))
                .collect(),
        ),
    }
}

fn validate_typed_compatibility(
    path: &Path,
    base: &[u8],
    local: &[u8],
    incoming: &[u8],
) -> Result<(), UpdateError> {
    let (Some(base), Some(local), Some(incoming)) = (
        parse_typed(path, base)?,
        parse_typed(path, local)?,
        parse_typed(path, incoming)?,
    ) else {
        return Ok(());
    };
    validate_compatible_nodes(path, Some(&base), Some(&local), Some(&incoming), "$", false)
}

fn validate_typed_merge_result(
    path: &Path,
    base: &[u8],
    local: &[u8],
    incoming: &[u8],
    merged: &[u8],
) -> Result<(), UpdateError> {
    let (Some(base), Some(local), Some(incoming), Some(merged)) = (
        parse_typed(path, base)?,
        parse_typed(path, local)?,
        parse_typed(path, incoming)?,
        parse_typed(path, merged)?,
    ) else {
        return Ok(());
    };
    validate_compatible_nodes(path, Some(&base), Some(&local), Some(&merged), "$", false)?;
    validate_compatible_nodes(
        path,
        Some(&base),
        Some(&incoming),
        Some(&merged),
        "$",
        false,
    )
}

fn validate_compatible_nodes(
    path: &Path,
    base: Option<&TypedValue>,
    local: Option<&TypedValue>,
    incoming: Option<&TypedValue>,
    location: &str,
    version_identifier: bool,
) -> Result<(), UpdateError> {
    if version_identifier && local != incoming {
        return Err(typed_compatibility_conflict(
            path,
            location,
            "incompatible schema/version identifiers",
        ));
    }
    match (local, incoming) {
        (Some(TypedValue::Mapping(local)), Some(TypedValue::Mapping(incoming))) => {
            let base = match base {
                Some(TypedValue::Mapping(base)) => Some(base),
                _ => None,
            };
            let keys = local
                .keys()
                .chain(incoming.keys())
                .cloned()
                .collect::<BTreeSet<_>>();
            for key in keys {
                let child_location = format!("{location}.{key}");
                validate_compatible_nodes(
                    path,
                    base.and_then(|values| values.get(&key)),
                    local.get(&key),
                    incoming.get(&key),
                    &child_location,
                    is_version_identifier(&key),
                )?;
            }
        }
        (Some(TypedValue::Sequence(local)), Some(TypedValue::Sequence(incoming))) => {
            let base = match base {
                Some(TypedValue::Sequence(base)) => Some(base.as_slice()),
                _ => None,
            };
            for (index, (local, incoming)) in local.iter().zip(incoming).enumerate() {
                let child_location = format!("{location}[{index}]");
                validate_compatible_nodes(
                    path,
                    base.and_then(|values| values.get(index)),
                    Some(local),
                    Some(incoming),
                    &child_location,
                    false,
                )?;
            }
        }
        (Some(TypedValue::Tagged(local_tag, local)), Some(TypedValue::Tagged(_, incoming))) => {
            let base = match base {
                Some(TypedValue::Tagged(base_tag, base)) if base_tag == local_tag => {
                    Some(base.as_ref())
                }
                _ => None,
            };
            validate_compatible_nodes(path, base, Some(local), Some(incoming), location, false)?;
        }
        _ => {}
    }
    if local == base || incoming == base || local == incoming {
        return Ok(());
    }
    let (Some(local), Some(incoming)) = (local, incoming) else {
        return Err(typed_compatibility_conflict(
            path,
            location,
            "conflicting structural add/delete changes",
        ));
    };
    if local.kind() != incoming.kind() {
        return Err(typed_compatibility_conflict(
            path,
            location,
            "conflicting field type changes",
        ));
    }
    match (local, incoming) {
        (TypedValue::Sequence(local), TypedValue::Sequence(incoming)) => {
            if local.len() != incoming.len() {
                return Err(typed_compatibility_conflict(
                    path,
                    location,
                    "conflicting sequence shape changes",
                ));
            }
        }
        (TypedValue::Tagged(local_tag, _), TypedValue::Tagged(incoming_tag, _))
            if local_tag != incoming_tag =>
        {
            return Err(typed_compatibility_conflict(
                path,
                location,
                "conflicting YAML tag changes",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn is_version_identifier(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| !matches!(character, '-' | '_'))
        .flat_map(char::to_lowercase)
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "$schema" | "schema" | "schemaversion" | "version"
    )
}

fn typed_compatibility_conflict(path: &Path, location: &str, reason: &str) -> UpdateError {
    UpdateError::Conflict(format!(
        "typed project projection is incompatible at {} ({location}): {reason}",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmodified_local_uses_incoming_exactly() {
        let result = three_way_merge(
            Path::new("skill.md"),
            Some(b"old\n"),
            Some(b"old\n"),
            Some(b"new\n"),
        )
        .unwrap();
        assert_eq!(result.bytes.as_deref(), Some(b"new\n".as_slice()));
        assert_eq!(result.disposition, MergeDisposition::IncomingReplace);
    }

    #[test]
    fn disjoint_hunks_merge_and_overlap_keeps_local() {
        let base = b"a\nbase-one\nmiddle\nbase-two\nz\n";
        let local = b"a\nlocal-one\nmiddle\nbase-two\nz\n";
        let incoming = b"a\nupstream-one\nmiddle\nupstream-two\nz\n";
        let result = three_way_merge(
            Path::new("skill.md"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap();
        assert_eq!(
            result.bytes.as_deref(),
            Some(b"a\nlocal-one\nmiddle\nupstream-two\nz\n".as_slice())
        );
        assert_eq!(result.omitted_incoming_hunks, 1);
        assert!(result.local_priority);
    }

    #[test]
    fn typed_merge_preserves_unknown_local_field_order() {
        let base = b"{\n  \"known\": 1,\n  \"tail\": true\n}\n";
        let local = b"{\n  \"known\": 1,\n  \"user-extra\": \"keep\",\n  \"tail\": true\n}\n";
        let incoming = b"{\n  \"known\": 2,\n  \"tail\": true\n}\n";
        let result = three_way_merge(
            Path::new("config.json"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap();
        assert_eq!(
            result.bytes.as_deref(),
            Some(
                b"{\n  \"known\": 2,\n  \"user-extra\": \"keep\",\n  \"tail\": true\n}\n"
                    .as_slice()
            )
        );
    }

    #[test]
    fn json_merge_rejects_divergent_schema_versions() {
        let base = b"{\n  \"schema_version\": 1,\n  \"value\": true\n}\n";
        let local = b"{\n  \"schema_version\": 2,\n  \"value\": true\n}\n";
        let incoming = b"{\n  \"schema_version\": 3,\n  \"value\": true\n}\n";
        let error = three_way_merge(
            Path::new("config.json"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.schema_version"));
    }

    #[test]
    fn json_merge_rejects_incoming_schema_change_against_local_edits() {
        let base = b"{\n  \"$schema\": \"v1\",\n  \"value\": true\n}\n";
        let local =
            b"{\n  \"$schema\": \"v1\",\n  \"user-extra\": \"keep\",\n  \"value\": true\n}\n";
        let incoming = b"{\n  \"$schema\": \"v2\",\n  \"value\": true\n}\n";
        let error = three_way_merge(
            Path::new("config.json"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.$schema"));
    }

    #[test]
    fn json_merge_rejects_nested_incoming_schema_change_against_root_local_edit() {
        let base = b"{\n  \"section\": {\n    \"schema_version\": 1\n  },\n  \"value\": true\n}\n";
        let local = b"{\n  \"user-extra\": \"keep\",\n  \"section\": {\n    \"schema_version\": 1\n  },\n  \"value\": true\n}\n";
        let incoming =
            b"{\n  \"section\": {\n    \"schema_version\": 2\n  },\n  \"value\": true\n}\n";
        let error = three_way_merge(
            Path::new("config.json"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.section.schema_version"));
    }

    #[test]
    fn json_merge_allows_one_sided_type_change_and_unknown_local_field() {
        let base = b"{\n  \"known\": 1,\n  \"tail\": true\n}\n";
        let local = b"{\n  \"known\": 1,\n  \"user-extra\": \"keep\",\n  \"tail\": true\n}\n";
        let incoming = b"{\n  \"known\": {\"nested\": true},\n  \"tail\": true\n}\n";
        let result = three_way_merge(
            Path::new("config.json"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap();
        assert_eq!(
            result.bytes.as_deref(),
            Some(
                b"{\n  \"known\": {\"nested\": true},\n  \"user-extra\": \"keep\",\n  \"tail\": true\n}\n"
                    .as_slice()
            )
        );
    }

    #[test]
    fn json_merge_allows_one_sided_sequence_shape_change() {
        let base = b"{\n  \"items\": [\n    1\n  ],\n  \"tail\": true\n}\n";
        let local =
            b"{\n  \"items\": [\n    1\n  ],\n  \"user-extra\": \"keep\",\n  \"tail\": true\n}\n";
        let incoming = b"{\n  \"items\": [\n    1,\n    2\n  ],\n  \"tail\": true\n}\n";
        let result = three_way_merge(
            Path::new("config.json"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap();
        assert_eq!(
            result.bytes.as_deref(),
            Some(
                b"{\n  \"items\": [\n    1,\n    2\n  ],\n  \"user-extra\": \"keep\",\n  \"tail\": true\n}\n"
                    .as_slice()
            )
        );
    }

    #[test]
    fn yaml_merge_rejects_conflicting_field_types() {
        let base = b"value: 1\ntail: true\n";
        let local = b"value:\n  nested: true\ntail: true\n";
        let incoming = b"value:\n  - item\ntail: true\n";
        let error = three_way_merge(
            Path::new("config.yaml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.value"));
    }

    #[test]
    fn yaml_merge_preserves_unknown_local_field_order() {
        let base = b"known: 1\ntail: true\n";
        let local = b"known: 1\nuser-extra: keep\ntail: true\n";
        let incoming = b"known: 2\ntail: true\n";
        let result = three_way_merge(
            Path::new("config.yaml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap();
        assert_eq!(
            result.bytes.as_deref(),
            Some(b"known: 2\nuser-extra: keep\ntail: true\n".as_slice())
        );
    }

    #[test]
    fn yaml_merge_rejects_nested_incoming_schema_change_against_root_local_edit() {
        let base = b"section:\n  schema_version: 1\nvalue: true\n";
        let local = b"user-extra: keep\nsection:\n  schema_version: 1\nvalue: true\n";
        let incoming = b"section:\n  schema_version: 2\nvalue: true\n";
        let error = three_way_merge(
            Path::new("config.yaml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.section.schema_version"));
    }

    #[test]
    fn yaml_merge_allows_one_sided_tag_change() {
        let base = b"value: !old item\ntail: true\n";
        let local = b"value: !old item\nuser-extra: keep\ntail: true\n";
        let incoming = b"value: !new item\ntail: true\n";
        let result = three_way_merge(
            Path::new("config.yaml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap();
        assert_eq!(
            result.bytes.as_deref(),
            Some(b"value: !new item\nuser-extra: keep\ntail: true\n".as_slice())
        );
    }

    #[test]
    fn toml_merge_rejects_divergent_schema_versions() {
        let base = b"schema-version = 1\nvalue = true\n";
        let local = b"schema-version = 2\nvalue = true\n";
        let incoming = b"schema-version = 3\nvalue = true\n";
        let error = three_way_merge(
            Path::new("config.toml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.schema-version"));
    }

    #[test]
    fn toml_merge_rejects_conflicting_primitive_types() {
        let base = b"count = 1\ntail = true\n";
        let local = b"count = \"many\"\ntail = true\n";
        let incoming = b"count = 1.5\ntail = true\n";
        let error = three_way_merge(
            Path::new("config.toml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.count"));
    }

    #[test]
    fn toml_merge_preserves_unknown_local_field_order() {
        let base = b"known = 1\ntail = true\n";
        let local = b"known = 1\nuser-extra = \"keep\"\ntail = true\n";
        let incoming = b"known = 2\ntail = true\n";
        let result = three_way_merge(
            Path::new("config.toml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap();
        assert_eq!(
            result.bytes.as_deref(),
            Some(b"known = 2\nuser-extra = \"keep\"\ntail = true\n".as_slice())
        );
    }

    #[test]
    fn toml_merge_rejects_nested_incoming_schema_change_against_root_local_edit() {
        let base = b"value = true\n\n[section]\nschema_version = 1\n";
        let local = b"value = true\nuser-extra = \"keep\"\n\n[section]\nschema_version = 1\n";
        let incoming = b"value = true\n\n[section]\nschema_version = 2\n";
        let error = three_way_merge(
            Path::new("config.toml"),
            Some(base),
            Some(local),
            Some(incoming),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
        assert!(error.to_string().contains("$.section.schema_version"));
    }

    #[test]
    fn occupied_path_without_base_fails_closed() {
        let error = three_way_merge(
            Path::new("skill.md"),
            None,
            Some(b"local\n"),
            Some(b"incoming\n"),
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Conflict(_)));
    }
}
