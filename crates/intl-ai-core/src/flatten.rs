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
pub fn set_nested(root: &mut Value, flat_key: &str, value: Value) {
    let mut segments: Vec<&str> = flat_key.split('.').collect();
    let Some(last) = segments.pop() else { return };
    let mut node = root;
    for seg in segments {
        if !node.is_object() {
            *node = Value::Object(Map::new());
        }
        node = node
            .as_object_mut()
            .expect("just ensured object")
            .entry(seg.to_string())
            .or_insert(Value::Null);
    }
    if !node.is_object() {
        *node = Value::Object(Map::new());
    }
    node.as_object_mut()
        .expect("just ensured object")
        .insert(last.to_string(), value);
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
        set_nested(&mut root, "a.b.c", json!("v"));
        assert_eq!(root["a"]["b"]["c"], "v");
        assert_eq!(root["keep"], 1);
        set_nested(&mut root, "deep", json!(2));
        assert_eq!(root["deep"], 2);
    }
}
