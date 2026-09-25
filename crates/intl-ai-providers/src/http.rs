//! OpenAI-compatible chat-completions transport (plan 5.1.9): strict
//! `json_schema` response_format, `Authorization: Bearer` injected,
//! `model_params` spread last so user params win over our defaults.

use intl_ai_core::config::HttpProvider;
use intl_ai_core::error::{Error, ErrorType, Result};
use intl_ai_core::transport::{
    JudgeRequest, Judgement, TranslateRequest, TranslateResponse, Translated, Transport,
};
use serde_json::{Map, Value, json};
use std::time::Duration;

use crate::prompt::{
    ADVERSARIAL_SYSTEM_PROMPT, judge_user_prompt, parse_judgements, parse_translations,
    system_prompt, user_prompt,
};
use crate::retry::attempt_with_retries;

const PER_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(300);
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const TEMPERATURE: f64 = 0.3;
const JUDGE_TEMPERATURE: f64 = 0.0;

/// Frozen response contract (plan 5.1.5).
const TRANSLATIONS_SCHEMA: &str = r#"{
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
const JUDGEMENTS_SCHEMA: &str = r#"{
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

pub struct HttpTransport {
    base_url: String,
    model: String,
    api_key: String,
    model_params: Map<String, Value>,
    max_retries: u32,
    agent: ureq::Agent,
}

impl HttpTransport {
    pub fn new(provider: &HttpProvider) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(PER_ATTEMPT_TIMEOUT))
            // We classify !2xx ourselves so the error keeps a body preview.
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            base_url: provider
                .base_url
                .clone()
                .unwrap_or_else(|| DEFAULT_BASE_URL.into())
                .trim_end_matches('/')
                .to_string(),
            model: provider.model.clone(),
            api_key: provider.api_key.clone(),
            model_params: provider.model_params.clone().unwrap_or_default(),
            max_retries: 3,
            agent,
        }
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    fn request_body(
        &self,
        system: &str,
        user: &str,
        schema_name: &str,
        schema_src: &str,
        temperature: f64,
    ) -> Value {
        let schema: Value = serde_json::from_str(schema_src).expect("static schema parses");
        let mut body = json!({
            "model": self.model,
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
        // modelParams spread last: user params win over our defaults
        // (plan 5.1.9), e.g. reasoning models overriding temperature.
        for (k, v) in &self.model_params {
            body[k] = v.clone();
        }
        body
    }

    fn attempt(
        &self,
        system: &str,
        user: &str,
        schema_name: &str,
        schema_src: &str,
        temperature: f64,
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut resp = self
            .agent
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .send_json(self.request_body(system, user, schema_name, schema_src, temperature))
            .map_err(|e| self.classify_transport_err(&e))?;

        let status = resp.status().as_u16();
        if status == 429 {
            let mut err = Error::transport(
                ErrorType::RateLimit,
                format!("openai: HTTP 429 {}", body_preview(&mut resp)),
            );
            if let Some(ms) = retry_after_ms(&resp) {
                err = err.with_retry_after(Duration::from_millis(ms));
            }
            return Err(err);
        }
        if status == 401 || status == 403 {
            return Err(Error::transport(
                ErrorType::Auth,
                format!(
                    "openai: HTTP {status} {} (check provider.api_key)",
                    body_preview(&mut resp)
                ),
            ));
        }
        if !(200..300).contains(&status) {
            return Err(Error::transport(
                ErrorType::Http,
                format!("openai: HTTP {status} {}", body_preview(&mut resp)),
            ));
        }
        let body: Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| Error::transport(ErrorType::ParseError, format!("openai: body: {e}")))?;
        let content = body
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                Error::transport(
                    ErrorType::ParseError,
                    "openai: missing choices[0].message.content".to_string(),
                )
            })?;
        // openai.ts parseResponse strips ``` fences before JSON.parse.
        let content = strip_code_fences(content);
        if content.trim().is_empty() {
            return Err(Error::transport(
                ErrorType::Empty,
                "empty response from model",
            ));
        }
        Ok(content)
    }

    fn classify_transport_err(&self, e: &ureq::Error) -> Error {
        match e {
            ureq::Error::Timeout(_) => Error::transport(ErrorType::Timeout, format!("openai: {e}")),
            other => Error::transport(ErrorType::Http, format!("openai: {other}")),
        }
    }
}

/// Retry-After / Retry-After-Ms header -> milliseconds. Delta-seconds
/// (and the -ms variant) are what OpenAI-compatible endpoints actually
/// send; an HTTP-date we can't parse just falls back to our own backoff.
fn retry_after_ms(resp: &ureq::http::Response<ureq::Body>) -> Option<u64> {
    let headers = resp.headers();
    for (name, scale) in [("retry-after-ms", 1.0), ("retry-after", 1000.0)] {
        if let Some(v) = headers.get(name).and_then(|v| v.to_str().ok()) {
            if let Ok(n) = v.trim().parse::<f64>() {
                return Some((n * scale) as u64);
            }
        }
    }
    None
}

fn strip_code_fences(content: &str) -> String {
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

fn body_preview(resp: &mut ureq::http::Response<ureq::Body>) -> String {
    let body = resp
        .body_mut()
        .read_to_string()
        .unwrap_or_else(|_| "<unreadable body>".into());
    let trimmed = body.trim();
    trimmed.chars().take(500).collect()
}

impl Transport for HttpTransport {
    fn id(&self) -> &str {
        "http"
    }

    fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse> {
        let system = system_prompt(req.locale_instruction.as_deref());
        let user = user_prompt(req);
        // Fetch + parse inside the attempt: parse_error and empty burn the
        // retry budget like transport errors (plan 5.1.10).
        attempt_with_retries(self.max_retries, || {
            let content = self.attempt(
                &system,
                &user,
                "translations",
                TRANSLATIONS_SCHEMA,
                TEMPERATURE,
            )?;
            let parsed = parse_translations(&content).map_err(|e| {
                Error::transport(
                    ErrorType::ParseError,
                    format!(
                        "openai: {e} (content: {})",
                        &content[..content.len().min(300)]
                    ),
                )
            })?;
            Ok(TranslateResponse {
                translations: parsed
                    .translations
                    .into_iter()
                    .map(|r| Translated {
                        key: r.key,
                        value: r.translated,
                    })
                    .collect(),
                model: self.model.clone(),
            })
        })
    }

    fn judge(&self, req: &JudgeRequest) -> Result<Vec<Judgement>> {
        let user = judge_user_prompt(&req.items, req.locale_instruction.as_deref());
        attempt_with_retries(self.max_retries, || {
            let content = self.attempt(
                ADVERSARIAL_SYSTEM_PROMPT,
                &user,
                "judgements",
                JUDGEMENTS_SCHEMA,
                JUDGE_TEMPERATURE,
            )?;
            let parsed = parse_judgements(&content).map_err(|e| {
                Error::transport(
                    ErrorType::ParseError,
                    format!(
                        "openai: judge: {e} (content: {})",
                        &content[..content.len().min(300)]
                    ),
                )
            })?;
            Ok(parsed
                .judgements
                .into_iter()
                .map(|r| Judgement {
                    key: r.key,
                    score: r.score,
                    reason: r.reason,
                    errors: r.errors,
                })
                .collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intl_ai_core::transport::TranslationEntry;

    #[test]
    fn body_shape_matches_ts() {
        let provider = HttpProvider {
            provider: "openai".into(),
            model: "gpt-5".into(),
            api_key: "sk-test".into(),
            base_url: None,
            model_params: Some(Map::from_iter([("temperature".to_string(), json!(0.9))])),
        };
        let t = HttpTransport::new(&provider);
        let body = t.request_body(
            "SYS",
            "USR",
            "translations",
            TRANSLATIONS_SCHEMA,
            TEMPERATURE,
        );
        assert_eq!(body["model"], "gpt-5");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "SYS");
        assert_eq!(body["messages"][1]["content"], "USR");
        assert_eq!(body["response_format"]["type"], "json_schema");
        assert_eq!(
            body["response_format"]["json_schema"]["schema"]["required"],
            json!(["translations"])
        );
        // modelParams spread last — wins over our temperature default.
        assert_eq!(body["temperature"], 0.9);
    }

    #[test]
    fn fence_stripping() {
        assert_eq!(
            strip_code_fences("```json\n{\"a\":1}\n```\n"),
            "{\"a\":1}\n"
        );
    }

    #[test]
    fn ignores_extra_fields_in_payload() {
        let content = r#"{"translations":[{"key":"a","translated":"x","confidence":0.9}]}"#;
        let parsed = parse_translations(content).unwrap();
        assert_eq!(parsed.translations[0].translated, "x");
    }

    #[test]
    fn translate_request_defaults() {
        let r = TranslateRequest {
            source_locale: "en".into(),
            target_locale: "fr".into(),
            entries: vec![TranslationEntry {
                key: "k".into(),
                source: "s".into(),
            }],
            ..Default::default()
        };
        let user = user_prompt(&r);
        assert!(user.contains("Preserve any placeholders"));
    }
}
