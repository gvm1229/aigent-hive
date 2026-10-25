//! A small transport document carries only supplied choices, never provenance or scope authority.
use crate::user_setup::SetupError;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Patch {
    schema_version: u8,
    kind: String,
    answers: Value,
}

pub(super) fn read(bytes: &[u8]) -> Result<Option<Value>, SetupError> {
    let Ok(value) = serde_yaml::from_slice::<Value>(bytes) else {
        return Ok(None);
    };
    if value["kind"] != "hive.setup-answer-patch" {
        return Ok(None);
    }
    let patch: Patch = serde_json::from_value(value)
        .map_err(|_| SetupError::Input("invalid setup answer patch".into()))?;
    if patch.schema_version != 1
        || patch.kind != "hive.setup-answer-patch"
        || patch
            .answers
            .as_object()
            .is_none_or(serde_json::Map::is_empty)
        || [
            "schema_version",
            "preference_decisions",
            "user_store_binding",
            "project_identity",
            "operation_id",
        ]
        .iter()
        .any(|key| patch.answers.get(*key).is_some())
    {
        return Err(SetupError::Input(
            "answer patches contain choices only; metadata and scope bindings are not answers"
                .into(),
        ));
    }
    Ok(Some(patch.answers))
}

pub(super) fn merge(base: &mut Value, supplied: &Value) -> Result<(), SetupError> {
    merge_at(base, supplied, 0)
}

fn merge_at(base: &mut Value, supplied: &Value, depth: usize) -> Result<(), SetupError> {
    if depth > 16 {
        return Err(SetupError::Input(
            "answer patch nesting exceeds its limit".into(),
        ));
    }
    if let (Some(base), Some(supplied)) = (base.as_object_mut(), supplied.as_object()) {
        for (key, value) in supplied {
            merge_at(base.entry(key).or_insert(Value::Null), value, depth + 1)?;
        }
    } else {
        *base = supplied.clone();
    }
    Ok(())
}

pub(super) fn require_initial(supplied: &Value, keys: &[&str]) -> Result<(), SetupError> {
    if keys.iter().any(|key| supplied.get(*key).is_none()) {
        return Err(SetupError::Input(format!(
            "initial answer patch requires: {}",
            keys.join(", ")
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn small_patch_preserves_other_choices_and_rejects_forged_provenance() {
        let mut base =
            json!({"wiki":{"enabled":true,"language":"ko"},"hosts":["codex"],"description":"keep"});
        let patch = read(br#"{"schema_version":1,"kind":"hive.setup-answer-patch","answers":{"wiki":{"enabled":false},"description":null}}"#).unwrap().unwrap();
        merge(&mut base, &patch).unwrap();
        assert_eq!(
            base,
            json!({"wiki":{"enabled":false,"language":"ko"},"hosts":["codex"],"description":null})
        );
        assert!(read(br#"{"schema_version":1,"kind":"hive.setup-answer-patch","answers":{"preference_decisions":{}}}"#).is_err());
        assert!(read(b"schema_version: 2\ninterface_language: ko\n")
            .unwrap()
            .is_none());
        assert!(read(b"not valid: [").unwrap().is_none());
    }
}
