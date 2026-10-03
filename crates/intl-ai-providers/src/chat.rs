//! Shared OpenAI-compatible chat-completions contract: the frozen
//! `json_schema` bodies and temperatures both the native HTTP transport
//! and the wasm binding emit. Pure request shaping — no IO, no `ureq` —
//! so the wasm build can produce byte-identical bodies and leave the
//! fetch itself to the host JS.

use serde_json::{Map, Value, json};

pub const TEMPERATURE: f64 = 0.3;
pub const JUDGE_TEMPERATURE: f64 = 0.0;

/// Frozen response contract (plan 5.1.5).
pub const TRANSLATIONS_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "translations": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "key": { "type": "string" },
          "translated": { "type": "string" }
        },
        "required": ["key", "translated"],
        "additionalProperties": false
      }
    }
  },
  "required": ["translations"],
  "additionalProperties": false
}"#;

/// Judge response contract (plan 5.2).
pub const JUDGEMENTS_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "judgements": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "key": { "type": "string" },
          "score": { "type": "number", "minimum": 0, "maximum": 1 },
          "reason": { "type": "string" },
          "errors": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["key", "score"],
        "additionalProperties": false
      }
    }
  },
  "required": ["judgements"],
  "additionalProperties": false
}"#;

/// Builds the chat-completions request body: `model`, system+user
/// messages, strict `json_schema` response_format, `temperature`, then
/// `model_params` spread last so user params win over our defaults
/// (plan 5.1.9), e.g. reasoning models overriding temperature.
pub fn build_chat_body(
    system: &str,
    user: &str,
    schema_name: &str,
    schema_src: &str,
    temperature: f64,
    model: &str,
    model_params: &Map<String, Value>,
) -> Value {
    let schema: Value = serde_json::from_str(schema_src).expect("static schema parses");
    let mut body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": { "name": schema_name, "schema": schema }
        },
        "temperature": temperature,
    });
    for (k, v) in model_params {
        body[k] = v.clone();
    }
    body
}

/// `openai.ts parseResponse` strips ``` fences before JSON.parse.
pub fn strip_code_fences(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with("```") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
