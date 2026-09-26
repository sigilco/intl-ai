use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Flat key -> string coercion table (plan 5.1.2): numbers/bools to their
/// string form, null to "null", arrays to compact JSON. Flat dot-keys are
/// indistinguishable from nested paths; writes always produce nested shape.
pub type FlatMap = BTreeMap<String, String>;

pub fn flatten(value: &Value) -> FlatMap {
    let mut out = BTreeMap::new();
    walk(value, "", &mut out);
    out
}

/// Leaves the JSON tree has that `flatten` cannot represent (ghost
/// keys): empty objects and dotted-path collisions where a later leaf
/// overwrites an earlier one. Nonzero means flattening lost data.
pub fn dropped_leaf_count(value: &Value) -> usize {
    leaf_count(value).saturating_sub(flatten(value).len())
}

fn leaf_count(value: &Value) -> usize {
    match value {
        Value::Object(map) if !map.is_empty() => map.values().map(leaf_count).sum(),
        _ => 1,
    }
}

fn walk(value: &Value, prefix: &str, out: &mut FlatMap) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                walk(v, &key, out);
            }
        }
        Value::Array(_) => {
            out.insert(
                prefix.to_string(),
                serde_json::to_string(value).unwrap_or_default(),
            );
        }
        Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        Value::Number(n) => {
            out.insert(prefix.to_string(), n.to_string());
        }
        Value::Bool(b) => {
            out.insert(prefix.to_string(), b.to_string());
        }
        Value::Null => {
            out.insert(prefix.to_string(), "null".to_string());
        }
    }
}

/// Sets `root[a][b][c] = value` for flat key `a.b.c`, creating intermediate
/// objects. An existing non-object node on the path is replaced; no TS-style
/// prototype guard is needed since JSON maps have no prototype semantics.
/// Returns the flat path of an existing value the write destroyed (a scalar
/// or array at an intermediate segment, or an object/array at the leaf), so
/// callers can surface it instead of silently clobbering human text.
/// Existing `null` nodes count as absent, not content.
pub fn set_nested(root: &mut Value, flat_key: &str, value: Value) -> Option<String> {
    let mut clobbered: Option<String> = None;
    let mut segments: Vec<&str> = flat_key.split('.').collect();
    let last = segments.pop()?;
    let mut node = root;
    // `prefix` is the flat path of `node` ("" at the root).
    let mut prefix = String::new();
    for seg in &segments {
        if !node.is_object() {
            if !node.is_null() && clobbered.is_none() {
                clobbered = Some(if prefix.is_empty() {
                    flat_key.to_string()
                } else {
                    prefix.clone()
                });
            }
            *node = Value::Object(Map::new());
        }
        if !prefix.is_empty() {
            prefix.push('.');
        }
        prefix.push_str(seg);
        node = node
            .as_object_mut()
            .expect("just ensured object")
            .entry(seg.to_string())
            .or_insert(Value::Null);
    }
    if !node.is_object() {
        if !node.is_null() && clobbered.is_none() {
            clobbered = Some(prefix);
        }
        *node = Value::Object(Map::new());
    }
    let obj = node.as_object_mut().expect("just ensured object");
    if clobbered.is_none() && obj.get(last).is_some_and(|v| v.is_object() || v.is_array()) {
        clobbered = Some(flat_key.to_string());
    }
    obj.insert(last.to_string(), value);
    clobbered
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flatten_coercion_table() {
        let v = json!({
            "a": {"b": "x", "n": 5, "t": true, "z": null},
            "arr": [1, "s"],
            "empty": {},
            "s": ""
        });
        let flat = flatten(&v);
        assert_eq!(flat["a.b"], "x");
        assert_eq!(flat["a.n"], "5");
        assert_eq!(flat["a.t"], "true");
        assert_eq!(flat["a.z"], "null");
        assert_eq!(flat["arr"], "[1,\"s\"]");
        assert!(!flat.contains_key("empty"));
        assert_eq!(flat["s"], "");
    }

    #[test]
    fn set_nested_creates_and_replaces() {
        let mut root = json!({"keep": 1, "a": "scalar"});
        assert_eq!(
            set_nested(&mut root, "a.b.c", json!("v")),
            Some("a".to_string())
        );
        assert_eq!(root["a"]["b"]["c"], "v");
        assert_eq!(root["keep"], 1);
        assert_eq!(set_nested(&mut root, "deep", json!(2)), None);
        assert_eq!(root["deep"], 2);
    }

    #[test]
    fn set_nested_reports_clobbered_values() {
        // Scalar at an intermediate segment.
        let mut root = json!({"a": "Menu"});
        assert_eq!(
            set_nested(&mut root, "a.b.c", json!("v")),
            Some("a".to_string())
        );
        // Array at the deepest intermediate.
        let mut root = json!({"a": {"b": [1, 2]}});
        assert_eq!(
            set_nested(&mut root, "a.b.c", json!("v")),
            Some("a.b".to_string())
        );
        // Object subtree at the leaf.
        let mut root = json!({"a": {"b": {"x": 1}}});
        assert_eq!(
            set_nested(&mut root, "a.b", json!("v")),
            Some("a.b".to_string())
        );
        // Null is treated as absent, not content.
        let mut root = json!({"a": null});
        assert_eq!(set_nested(&mut root, "a.b", json!("v")), None);
        // Scalar leaf overwrite (normal fill) is not a clobber.
        let mut root = json!({"a": {"b": "old"}});
        assert_eq!(set_nested(&mut root, "a.b", json!("new")), None);
    }
}
