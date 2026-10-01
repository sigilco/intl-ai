//! wasm-bindgen surface for the intl-ai pipeline. Binds only the pure
//! leaves — flatten/set_nested, prompt building, response parsing,
//! rule checks — while the host JS owns all IO: file reads, the fetch
//! loop, and retry/backoff. Filesystem-bound modules (`fill`, `check`,
//! lockfile, spec/exec checks) and every provider transport stay out of
//! this build entirely.

use intl_ai_checks::builtin;
use intl_ai_core::check::{CheckCtx, CheckItem};
use intl_ai_core::flatten::{self, FlatMap};
use intl_ai_core::transport::{JudgeItem, TranslateRequest, TranslationEntry};
use intl_ai_formats::FileFormat;
use intl_ai_providers::chat::{
    self, JUDGE_TEMPERATURE, JUDGEMENTS_SCHEMA, TEMPERATURE, TRANSLATIONS_SCHEMA,
};
use intl_ai_providers::payload::payload_or_self;
use intl_ai_providers::prompt::{
    self, ADVERSARIAL_SYSTEM_PROMPT, judge_user_prompt, system_prompt, user_prompt,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

fn js_err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

fn from_js<T: for<'de> Deserialize<'de>>(value: JsValue) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(value).map_err(js_err)
}

/// JSON-compatible serialization: maps and structs become plain JS
/// objects (not `Map`), matching the documented API surface.
fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(js_err)
}

fn format_of(ext: Option<&str>) -> Result<FileFormat, JsError> {
    match ext {
        None | Some("auto") => Ok(FileFormat::Json),
        Some("json") => Ok(FileFormat::Json),
        Some("yaml") | Some("yml") => Ok(FileFormat::Yaml),
        Some(other) => Err(JsError::new(&format!(
            "unsupported locale format '{other}' (expected json|yaml)"
        ))),
    }
}

/// Flatten a locale file's contents (JSON or YAML text) into a flat
/// `key -> string` map.
///
/// `flatten(sourceJson): Record<string, string>`
#[wasm_bindgen(js_name = flatten)]
pub fn flatten_js(source: &str) -> Result<JsValue, JsError> {
    let value = intl_ai_formats::parse_auto(source).map_err(js_err)?;
    to_js(&flatten::flatten(&value))
}

/// Source keys with no counterpart in the target map, sorted — the set
/// `fill` would translate for a fresh or partial locale file.
///
/// `missingKeys(sourceFlat, targetFlat | null | undefined): string[]`
#[wasm_bindgen(js_name = missingKeys)]
pub fn missing_keys(source_flat: JsValue, target_flat: JsValue) -> Result<JsValue, JsError> {
    let source: FlatMap = from_js(source_flat)?;
    let target: FlatMap = if target_flat.is_null() || target_flat.is_undefined() {
        FlatMap::new()
    } else {
        from_js(target_flat)?
    };
    let missing: Vec<String> = source
        .keys()
        .filter(|k| !target.contains_key(*k))
        .cloned()
        .collect();
    to_js(&missing)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslateBodyRequest {
    source_locale: String,
    target_locale: String,
    entries: Vec<Entry>,
    #[serde(default)]
    glossary: BTreeMap<String, String>,
    locale_instruction: Option<String>,
    #[serde(default)]
    feedback: BTreeMap<String, String>,
    syntax_hint: Option<String>,
    model: String,
    #[serde(default)]
    model_params: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    key: String,
    source: String,
}

/// The exact OpenAI-compatible chat body the native `http` transport
/// would send for this translation batch: system+user prompts, strict
/// `json_schema` response_format, temperature 0.3, `modelParams`
/// spread last. The host JS posts it to `{baseUrl}/chat/completions`.
///
/// `buildTranslateBody(req): object`
#[wasm_bindgen(js_name = buildTranslateBody)]
pub fn build_translate_body(req: JsValue) -> Result<JsValue, JsError> {
    let req: TranslateBodyRequest = from_js(req)?;
    let translate = TranslateRequest {
        source_locale: req.source_locale,
        target_locale: req.target_locale,
        entries: req
            .entries
            .into_iter()
            .map(|e| TranslationEntry {
                key: e.key,
                source: e.source,
            })
            .collect(),
        glossary: req.glossary,
        locale_instruction: req.locale_instruction,
        feedback: req.feedback,
        syntax_hint: req.syntax_hint,
    };
    let body = chat::build_chat_body(
        &system_prompt(translate.locale_instruction.as_deref()),
        &user_prompt(&translate),
        "translations",
        TRANSLATIONS_SCHEMA,
        TEMPERATURE,
        &req.model,
        &req.model_params,
    );
    to_js(&body)
}

/// `choices[0].message.content` -> `key -> translated` map. Tolerates
/// upstreams that ignore `json_schema` and wrap the payload in markdown
/// fences or extra prose (fenced-block then balanced-brace extraction,
/// same as the command transport).
///
/// `parseTranslations(content): Record<string, string>`
#[wasm_bindgen(js_name = parseTranslations)]
pub fn parse_translations_js(content: &str) -> Result<JsValue, JsError> {
    let cleaned = chat::strip_code_fences(content);
    let payload = payload_or_self(&cleaned);
    let parsed = prompt::parse_translations(&payload).map_err(|e| {
        js_err(format!(
            "{e} (content: {})",
            &content[..content.len().min(300)]
        ))
    })?;
    let map: BTreeMap<String, String> = parsed
        .translations
        .into_iter()
        .map(|r| (r.key, r.translated))
        .collect();
    to_js(&map)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JudgeBodyRequest {
    items: Vec<JudgeEntry>,
    locale_instruction: Option<String>,
    model: String,
    #[serde(default)]
    model_params: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JudgeEntry {
    key: String,
    locale: String,
    source: String,
    translation: String,
}

/// The adversarial judge batch body (system prompt, per-item
/// source/translation block, `judgements` json_schema, temperature 0).
/// Same fetch target as `buildTranslateBody` — JS scores the filled
/// file in a second request instead of wiring a transport callback.
///
/// `buildJudgeBody(req): object`
#[wasm_bindgen(js_name = buildJudgeBody)]
pub fn build_judge_body(req: JsValue) -> Result<JsValue, JsError> {
    let req: JudgeBodyRequest = from_js(req)?;
    let user = judge_user_prompt(
        &req.items
            .into_iter()
            .map(|i| JudgeItem {
                key: i.key,
                locale: i.locale,
                source: i.source,
                translation: i.translation,
            })
            .collect::<Vec<_>>(),
        req.locale_instruction.as_deref(),
    );
    let body = chat::build_chat_body(
        ADVERSARIAL_SYSTEM_PROMPT,
        &user,
        "judgements",
        JUDGEMENTS_SCHEMA,
        JUDGE_TEMPERATURE,
        &req.model,
        &req.model_params,
    );
    to_js(&body)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JudgementOut {
    key: String,
    score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    errors: Vec<String>,
}

/// `choices[0].message.content` -> per-key adversarial scores, with the
/// same lenient payload extraction as `parseTranslations`.
///
/// `parseJudgements(content): [{key, score, reason?, errors}]`
#[wasm_bindgen(js_name = parseJudgements)]
pub fn parse_judgements_js(content: &str) -> Result<JsValue, JsError> {
    let cleaned = chat::strip_code_fences(content);
    let payload = payload_or_self(&cleaned);
    let parsed = prompt::parse_judgements(&payload).map_err(|e| {
        js_err(format!(
            "judge: {e} (content: {})",
            &content[..content.len().min(300)]
        ))
    })?;
    let out: Vec<JudgementOut> = parsed
        .judgements
        .into_iter()
        .map(|j| JudgementOut {
            key: j.key,
            score: j.score,
            reason: j.reason,
            errors: j.errors,
        })
        .collect();
    to_js(&out)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckItemIn {
    key: String,
    source: Option<String>,
    target: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckCtxIn {
    source_locale: String,
    target_locale: String,
    locale_instruction: Option<String>,
}

/// Run builtin rule checks against flattened items. Supported ids are
/// the pure builtins: `icu`, `placeholder-parity`, `dialect:<locale>`.
/// `judge` is rejected here — it needs a provider round trip, which JS
/// performs via `buildJudgeBody`/`parseJudgements`.
///
/// `runChecks(items, checkIds, ctx): findings[]`
#[wasm_bindgen(js_name = runChecks)]
pub fn run_checks(items: JsValue, check_ids: JsValue, ctx: JsValue) -> Result<JsValue, JsError> {
    let items: Vec<CheckItemIn> = from_js(items)?;
    let check_ids: Vec<String> = from_js(check_ids)?;
    let ctx: CheckCtxIn = from_js(ctx)?;

    let items: Vec<CheckItem> = items
        .into_iter()
        .map(|i| CheckItem {
            key: i.key,
            source: i.source,
            target: i.target,
        })
        .collect();
    let check_ctx = CheckCtx {
        source_locale: &ctx.source_locale,
        target_locale: &ctx.target_locale,
        transport: None,
        locale_instruction: ctx.locale_instruction.as_deref(),
    };

    let mut findings = Vec::new();
    for id in &check_ids {
        if id == "judge" {
            return Err(JsError::new(
                "judge requires a provider round trip; use buildJudgeBody + parseJudgements",
            ));
        }
        let check = builtin(id).map_err(js_err)?;
        let out = check.run(&check_ctx, &items).map_err(js_err)?;
        findings.extend(out.findings);
    }
    to_js(&findings)
}

/// Merge flat `key -> value` entries into a locale tree, starting from
/// the existing target file contents (or an empty object), and
/// serialize the filled file back out — the same `set_nested` +
/// canonical writer `fill` uses.
///
/// `unflatten(targetJsonOrYaml | null | undefined, map, format?): string`
#[wasm_bindgen(js_name = unflatten)]
pub fn unflatten(target: JsValue, map: JsValue, format: JsValue) -> Result<JsValue, JsError> {
    let mut root = if target.is_null() || target.is_undefined() {
        Value::Object(Map::new())
    } else {
        let text: String = from_js(target)?;
        intl_ai_formats::parse_auto(&text).map_err(js_err)?
    };
    if !root.is_object() {
        return Err(JsError::new(
            "existing locale file must contain a JSON object at its root",
        ));
    }
    let map: FlatMap = from_js(map)?;
    for (key, value) in map {
        flatten::set_nested(&mut root, &key, Value::String(value));
    }
    let format = format_of(from_js::<Option<String>>(format)?.as_deref())?;
    let bytes = match format {
        FileFormat::Json => intl_ai_formats::json::serialize(&root),
        FileFormat::Yaml => {
            intl_ai_formats::yaml::serialize(Path::new("locale.yaml"), &root).map_err(js_err)?
        }
    };
    let text = String::from_utf8(bytes)
        .map_err(|e| JsError::new(&format!("serialized locale is not utf-8: {e}")))?;
    to_js(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn translate_body() -> TranslateBodyRequest {
        TranslateBodyRequest {
            source_locale: "en".into(),
            target_locale: "fr".into(),
            entries: vec![Entry {
                key: "nav.home".into(),
                source: "Home".into(),
            }],
            glossary: BTreeMap::new(),
            locale_instruction: None,
            feedback: BTreeMap::new(),
            syntax_hint: None,
            model: "illo-demo".into(),
            model_params: Map::new(),
        }
    }

    #[test]
    fn translate_body_matches_http_shape() {
        let req = translate_body();
        let translate = TranslateRequest {
            source_locale: req.source_locale.clone(),
            target_locale: req.target_locale.clone(),
            entries: vec![TranslationEntry {
                key: "nav.home".into(),
                source: "Home".into(),
            }],
            ..Default::default()
        };
        let body = chat::build_chat_body(
            &system_prompt(None),
            &user_prompt(&translate),
            "translations",
            TRANSLATIONS_SCHEMA,
            TEMPERATURE,
            &req.model,
            &req.model_params,
        );
        assert_eq!(body["model"], "illo-demo");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["response_format"]["type"], "json_schema");
        assert_eq!(body["temperature"], json!(0.3));
    }

    #[test]
    fn formats_parse_auto_handles_yaml() {
        let v = intl_ai_formats::parse_auto("greeting: Salut\n").unwrap();
        assert_eq!(v["greeting"], json!("Salut"));
    }

    #[test]
    fn parse_translations_strips_fences() {
        let cleaned = chat::strip_code_fences("```json\n{\"translations\":[]}\n```\n");
        let payload = payload_or_self(&cleaned);
        let parsed = prompt::parse_translations(&payload).unwrap();
        assert!(parsed.translations.is_empty());
    }
}
