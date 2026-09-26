//! Minimal JSON edits for native hook arrays without reserializing foreign settings.
//! Only caller-selected entries and their required separators may change.

use serde_json::Value;
use std::collections::BTreeSet;
use std::ops::Range;

#[derive(Debug)]
enum Kind {
    Object(Vec<Member>),
    Array(Vec<Node>),
    Scalar,
}
#[derive(Debug)]
struct Node {
    range: Range<usize>,
    kind: Kind,
}
#[derive(Debug)]
struct Member {
    key: String,
    start: usize,
    value: Node,
}

fn whitespace(bytes: &[u8], position: &mut usize) {
    while bytes.get(*position).is_some_and(u8::is_ascii_whitespace) {
        *position += 1;
    }
}

fn string_end(bytes: &[u8], position: &mut usize) -> Result<(), &'static str> {
    if bytes.get(*position) != Some(&b'"') {
        return Err("expected JSON string");
    }
    *position += 1;
    while let Some(byte) = bytes.get(*position) {
        *position += 1;
        match byte {
            b'"' => return Ok(()),
            b'\\' => *position += 1,
            _ => {}
        }
    }
    Err("unterminated JSON string")
}

fn node(bytes: &[u8], position: &mut usize, depth: usize) -> Result<Node, &'static str> {
    if depth > 64 {
        return Err("hook settings nesting exceeds limit");
    }
    whitespace(bytes, position);
    let start = *position;
    let kind = match bytes.get(*position) {
        Some(b'{') => {
            *position += 1;
            let mut members = Vec::new();
            let mut keys = BTreeSet::new();
            loop {
                whitespace(bytes, position);
                if bytes.get(*position) == Some(&b'}') {
                    *position += 1;
                    break;
                }
                let key_start = *position;
                string_end(bytes, position)?;
                let key: String = serde_json::from_slice(&bytes[key_start..*position])
                    .map_err(|_| "invalid JSON key")?;
                if !keys.insert(key.clone()) {
                    return Err("duplicate setting key");
                }
                whitespace(bytes, position);
                if bytes.get(*position) != Some(&b':') {
                    return Err("missing JSON colon");
                }
                *position += 1;
                let value = node(bytes, position, depth + 1)?;
                members.push(Member {
                    key,
                    start: key_start,
                    value,
                });
                whitespace(bytes, position);
                match bytes.get(*position) {
                    Some(b',') => *position += 1,
                    Some(b'}') => {}
                    _ => return Err("invalid JSON object"),
                }
            }
            Kind::Object(members)
        }
        Some(b'[') => {
            *position += 1;
            let mut entries = Vec::new();
            loop {
                whitespace(bytes, position);
                if bytes.get(*position) == Some(&b']') {
                    *position += 1;
                    break;
                }
                entries.push(node(bytes, position, depth + 1)?);
                whitespace(bytes, position);
                match bytes.get(*position) {
                    Some(b',') => *position += 1,
                    Some(b']') => {}
                    _ => return Err("invalid JSON array"),
                }
            }
            Kind::Array(entries)
        }
        Some(b'"') => {
            string_end(bytes, position)?;
            Kind::Scalar
        }
        Some(_) => {
            while bytes
                .get(*position)
                .is_some_and(|byte| !byte.is_ascii_whitespace() && !b",]}".contains(byte))
            {
                *position += 1;
            }
            Kind::Scalar
        }
        None => return Err("incomplete JSON"),
    };
    Ok(Node {
        range: start..*position,
        kind,
    })
}

fn parse(bytes: &[u8]) -> Result<Node, &'static str> {
    if bytes.len() > 1024 * 1024 {
        return Err("hook settings exceed limit");
    }
    serde_json::from_slice::<Value>(bytes).map_err(|_| "hook settings require valid JSON")?;
    let root = node(bytes, &mut 0, 0)?;
    if !matches!(root.kind, Kind::Object(_)) {
        return Err("hook settings must be an object");
    }
    Ok(root)
}

fn find<'a>(root: &'a Node, path: &[&str]) -> Option<&'a Node> {
    let Some((key, tail)) = path.split_first() else {
        return Some(root);
    };
    let Kind::Object(members) = &root.kind else {
        return None;
    };
    members
        .iter()
        .find(|member| member.key == *key)
        .and_then(|member| find(&member.value, tail))
}

fn splice(bytes: &[u8], range: Range<usize>, insertion: &[u8]) -> Vec<u8> {
    [&bytes[..range.start], insertion, &bytes[range.end..]].concat()
}

fn remove_range(bytes: &[u8], ranges: &[Range<usize>], index: usize) -> Range<usize> {
    let current = ranges[index].clone();
    if index > 0 {
        let start = ranges[index - 1].end;
        let comma = bytes[start..current.start]
            .iter()
            .position(|byte| *byte == b',')
            .expect("parsed separator");
        (start + comma)..current.end
    } else if ranges.len() > 1 {
        let comma = bytes[current.end..ranges[1].start]
            .iter()
            .position(|byte| *byte == b',')
            .expect("parsed separator");
        current.start..(current.end + comma + 1)
    } else {
        current
    }
}

/// Inspect existence without changing or normalizing any source bytes.
///
/// # Errors
/// Rejects malformed, oversized, deeply nested or duplicate-key settings.
pub fn contains_path(bytes: &[u8], path: &[&str]) -> Result<bool, &'static str> {
    Ok(find(&parse(bytes)?, path).is_some())
}

/// Insert, replace or remove exactly one approved native hook array entry.
///
/// # Errors
/// Rejects ambiguous/missing prior entries, duplicate keys, and incompatible containers.
pub fn edit_entry(
    bytes: &[u8],
    path: &[&str],
    prior: Option<&Value>,
    desired: Option<&Value>,
) -> Result<Vec<u8>, &'static str> {
    if path.is_empty() || path.len() > 4 {
        return Err("invalid hook settings path");
    }
    let root = parse(bytes)?;
    if let Some(array) = find(&root, path) {
        let Kind::Array(entries) = &array.kind else {
            return Err("hook event must be an array");
        };
        let matches = entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let value: Value = serde_json::from_slice(&bytes[entry.range.clone()]).ok()?;
                (Some(&value) == prior.or(desired)).then_some(index)
            })
            .collect::<Vec<_>>();
        if matches.len() > 1 {
            return Err("ambiguous duplicate hook entry");
        }
        if let Some(index) = matches.first().copied() {
            if prior.is_none() || prior == desired {
                return Ok(bytes.to_vec());
            }
            let insertion = desired
                .map(serde_json::to_vec)
                .transpose()
                .map_err(|_| "cannot encode hook")?
                .unwrap_or_default();
            let range = if desired.is_none() {
                remove_range(
                    bytes,
                    &entries
                        .iter()
                        .map(|entry| entry.range.clone())
                        .collect::<Vec<_>>(),
                    index,
                )
            } else {
                entries[index].range.clone()
            };
            return Ok(splice(bytes, range, &insertion));
        }
        if prior.is_some() {
            return Err("approved hook entry changed or disappeared");
        }
        let Some(desired) = desired else {
            return Ok(bytes.to_vec());
        };
        let mut insertion = if entries.is_empty() {
            Vec::new()
        } else {
            vec![b',']
        };
        insertion.extend(serde_json::to_vec(desired).map_err(|_| "cannot encode hook")?);
        return Ok(splice(
            bytes,
            (array.range.end - 1)..(array.range.end - 1),
            &insertion,
        ));
    }
    if prior.is_some() {
        return Err("approved hook event disappeared");
    }
    let Some(desired) = desired else {
        return Ok(bytes.to_vec());
    };
    let mut parent = &root;
    for (index, key) in path.iter().enumerate() {
        let Kind::Object(members) = &parent.kind else {
            return Err("hook settings parent must be an object");
        };
        if let Some(member) = members.iter().find(|member| member.key == *key) {
            parent = &member.value;
            continue;
        }
        let mut value = Value::Array(vec![desired.clone()]);
        for component in path[index + 1..].iter().rev() {
            value = serde_json::json!({*component:value});
        }
        let mut insertion = if members.is_empty() {
            Vec::new()
        } else {
            vec![b',']
        };
        insertion.extend(serde_json::to_vec(key).map_err(|_| "cannot encode key")?);
        insertion.push(b':');
        insertion.extend(serde_json::to_vec(&value).map_err(|_| "cannot encode hook")?);
        return Ok(splice(
            bytes,
            (parent.range.end - 1)..(parent.range.end - 1),
            &insertion,
        ));
    }
    Err("invalid hook settings shape")
}

/// Remove a caller-owned empty container after revoking its last owned entry.
/// Never removes a container that another writer populated.
///
/// # Errors
/// Rejects invalid settings; ownership of `path` must come from the installation receipt.
pub fn prune_empty(bytes: &[u8], path: &[&str]) -> Result<Vec<u8>, &'static str> {
    let root = parse(bytes)?;
    let Some((key, parent_path)) = path.split_last() else {
        return Err("cannot prune settings root");
    };
    let Some(parent) = find(&root, parent_path) else {
        return Ok(bytes.to_vec());
    };
    let Kind::Object(members) = &parent.kind else {
        return Err("hook parent changed type");
    };
    let Some(index) = members.iter().position(|member| member.key == *key) else {
        return Ok(bytes.to_vec());
    };
    let empty = match &members[index].value.kind {
        Kind::Array(items) => items.is_empty(),
        Kind::Object(items) => items.is_empty(),
        Kind::Scalar => false,
    };
    if !empty {
        return Ok(bytes.to_vec());
    }
    let ranges = members
        .iter()
        .map(|member| member.start..member.value.range.end)
        .collect::<Vec<_>>();
    Ok(splice(bytes, remove_range(bytes, &ranges, index), &[]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn install_and_revoke_preserve_foreign_bytes_and_empty_container_ownership() {
        for original in [
            b"{\r\n  \"foreign\" : {\"keep\": [1, 2]}\r\n}".as_slice(),
            b"{ \"hooks\": {\"PreToolUse\": [  ]} }",
        ] {
            let entry = json!({"matcher":"Write","hooks":[{"type":"command","command":"hive policy hook"}]});
            let path = ["hooks", "PreToolUse"];
            let had_hooks = contains_path(original, &path[..1]).expect("parse");
            let had_event = contains_path(original, &path).expect("parse");
            let installed = edit_entry(original, &path, None, Some(&entry)).expect("install");
            assert_eq!(
                edit_entry(&installed, &path, None, Some(&entry)).expect("replay"),
                installed
            );
            let mut revoked = edit_entry(&installed, &path, Some(&entry), None).expect("revoke");
            if !had_event {
                revoked = prune_empty(&revoked, &path).expect("event");
            }
            if !had_hooks {
                revoked = prune_empty(&revoked, &path[..1]).expect("hooks");
            }
            assert_eq!(revoked, original);
        }
    }

    #[test]
    fn concurrent_foreign_entries_survive_and_ambiguous_definitions_fail() {
        let old = json!({"command":"hive"});
        let foreign = json!({"command":"foreign"});
        let installed =
            edit_entry(b"{}", &["hooks", "PreToolUse"], None, Some(&old)).expect("install");
        let edited = edit_entry(&installed, &["hooks", "PreToolUse"], None, Some(&foreign))
            .expect("foreign edit");
        let revoked =
            edit_entry(&edited, &["hooks", "PreToolUse"], Some(&old), None).expect("revoke");
        let value: Value = serde_json::from_slice(&revoked).expect("valid JSON");
        assert_eq!(value["hooks"]["PreToolUse"], json!([foreign]));
        assert!(edit_entry(
            br#"{"hooks":{},"hooks":{}}"#,
            &["hooks", "PreToolUse"],
            None,
            Some(&old)
        )
        .is_err());
        assert!(edit_entry(&installed, &["hooks", "PreToolUse"], Some(&foreign), None).is_err());
    }
}
