//! wasm-bindgen surface for the intl-ai pipeline. Binds the pure
//! leaves — flatten/set_nested, prompt building, response parsing,
//! rule checks — plus two streaming drivers (`fillRun`, `checkRun`)
//! that run the demo cut of the pipelines and forward the core
//! `ProgressEvent` stream to a host callback. The host JS owns all IO:
//! file reads, the fetch loop the drivers call back into, and
//! retry/backoff. Filesystem-bound modules (`fill`, `check`, lockfile,
//! spec/exec checks) and every provider transport stay out of this
//! build entirely.

use intl_ai_checks::builtin;
use intl_ai_core::check::{CheckCtx, CheckItem};
use intl_ai_core::diff::{CheckFinding, FindingKind};
use intl_ai_core::error::ErrorType;
use intl_ai_core::fill::FillFailure;
use intl_ai_core::flatten::{self, FlatMap};
use intl_ai_core::lockfile::Origin;
use intl_ai_core::progress::{KeyOutcome, Pipeline, ProgressEvent};
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
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

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

fn translate_body(req: TranslateBodyRequest) -> Value {
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
    chat::build_chat_body(
        &system_prompt(translate.locale_instruction.as_deref()),
        &user_prompt(&translate),
        "translations",
        TRANSLATIONS_SCHEMA,
        TEMPERATURE,
        &req.model,
        &req.model_params,
    )
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
    to_js(&translate_body(req))
}

fn parse_translations_map(content: &str) -> Result<BTreeMap<String, String>, JsError> {
    let cleaned = chat::strip_code_fences(content);
    let payload = payload_or_self(&cleaned);
    let parsed = prompt::parse_translations(&payload).map_err(|e| {
        js_err(format!(
            "{e} (content: {})",
            &content[..content.len().min(300)]
        ))
    })?;
    Ok(parsed
        .translations
        .into_iter()
        .map(|r| (r.key, r.translated))
        .collect())
}

/// `choices[0].message.content` -> `key -> translated` map. Tolerates
/// upstreams that ignore `json_schema` and wrap the payload in markdown
/// fences or extra prose (fenced-block then balanced-brace extraction,
/// same as the command transport).
///
/// `parseTranslations(content): Record<string, string>`
#[wasm_bindgen(js_name = parseTranslations)]
pub fn parse_translations_js(content: &str) -> Result<JsValue, JsError> {
    to_js(&parse_translations_map(content)?)
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

fn judge_body(req: JudgeBodyRequest) -> Value {
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
    chat::build_chat_body(
        ADVERSARIAL_SYSTEM_PROMPT,
        &user,
        "judgements",
        JUDGEMENTS_SCHEMA,
        JUDGE_TEMPERATURE,
        &req.model,
        &req.model_params,
    )
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
    to_js(&judge_body(req))
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

fn parse_judgements_vec(content: &str) -> Result<Vec<JudgementOut>, JsError> {
    let cleaned = chat::strip_code_fences(content);
    let payload = payload_or_self(&cleaned);
    let parsed = prompt::parse_judgements(&payload).map_err(|e| {
        js_err(format!(
            "judge: {e} (content: {})",
            &content[..content.len().min(300)]
        ))
    })?;
    Ok(parsed
        .judgements
        .into_iter()
        .map(|j| JudgementOut {
            key: j.key,
            score: j.score,
            reason: j.reason,
            errors: j.errors,
        })
        .collect())
}

/// `choices[0].message.content` -> per-key adversarial scores, with the
/// same lenient payload extraction as `parseTranslations`.
///
/// `parseJudgements(content): [{key, score, reason?, errors}]`
#[wasm_bindgen(js_name = parseJudgements)]
pub fn parse_judgements_js(content: &str) -> Result<JsValue, JsError> {
    to_js(&parse_judgements_vec(content)?)
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

/// Run one builtin check id against the item batch. `judge` is
/// rejected — it needs a provider round trip.
fn run_one_check(
    id: &str,
    ctx: &CheckCtx,
    items: &[CheckItem],
) -> Result<Vec<CheckFinding>, JsError> {
    if id == "judge" {
        return Err(JsError::new(
            "judge requires a provider round trip; use buildJudgeBody + parseJudgements",
        ));
    }
    let check = builtin(id).map_err(js_err)?;
    Ok(check.run(ctx, items).map_err(js_err)?.findings)
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
        findings.extend(run_one_check(id, &check_ctx, &items)?);
    }
    to_js(&findings)
}

fn unflatten_impl(mut root: Value, map: FlatMap, format: Option<&str>) -> Result<String, JsError> {
    if !root.is_object() {
        return Err(JsError::new(
            "existing locale file must contain a JSON object at its root",
        ));
    }
    for (key, value) in map {
        flatten::set_nested(&mut root, &key, Value::String(value));
    }
    let bytes = match format_of(format)? {
        FileFormat::Json => intl_ai_formats::json::serialize(&root),
        FileFormat::Yaml => {
            intl_ai_formats::yaml::serialize(Path::new("locale.yaml"), &root).map_err(js_err)?
        }
    };
    String::from_utf8(bytes)
        .map_err(|e| JsError::new(&format!("serialized locale is not utf-8: {e}")))
}

/// Merge flat `key -> value` entries into a locale tree, starting from
/// the existing target file contents (or an empty object), and
/// serialize the filled file back out — the same `set_nested` +
/// canonical writer `fill` uses.
///
/// `unflatten(targetJsonOrYaml | null | undefined, map, format?): string`
#[wasm_bindgen(js_name = unflatten)]
pub fn unflatten(target: JsValue, map: JsValue, format: JsValue) -> Result<JsValue, JsError> {
    let root = if target.is_null() || target.is_undefined() {
        Value::Object(Map::new())
    } else {
        let text: String = from_js(target)?;
        intl_ai_formats::parse_auto(&text).map_err(js_err)?
    };
    let map: FlatMap = from_js(map)?;
    let format = from_js::<Option<String>>(format)?;
    to_js(&unflatten_impl(root, map, format.as_deref())?)
}

// ---------------------------------------------------------------------------
// Streaming drivers: `runFill` / `runCheck` run the demo cut of each
// pipeline inside wasm and forward the core `ProgressEvent` stream to a
// host callback. The host owns IO the same way it does for the leaf
// API: it passes a transport function `(chatBody) => Promise<content>`
// that posts the request and resolves `choices[0].message.content`.

fn js_message(value: &JsValue) -> String {
    if let Some(s) = value.as_string() {
        return s;
    }
    js_sys::Reflect::get(value, &JsValue::from_str("message"))
        .ok()
        .and_then(|m| m.as_string())
        .unwrap_or_else(|| format!("{value:?}"))
}

fn js_function(value: JsValue, name: &str) -> Result<js_sys::Function, JsError> {
    if !value.is_function() {
        return Err(JsError::new(&format!("{name} must be a function")));
    }
    Ok(value.unchecked_into::<js_sys::Function>())
}

/// Call the host transport: `(body) => Promise<content>` (a bare string
/// result works too). The awaited value is the response's
/// `choices[0].message.content`.
async fn call_provider(transport: &js_sys::Function, body: &JsValue) -> Result<String, String> {
    let pending = transport
        .call1(&JsValue::NULL, body)
        .map_err(|e| js_message(&e))?;
    let value = JsFuture::from(js_sys::Promise::resolve(&pending))
        .await
        .map_err(|e| js_message(&e))?;
    value
        .as_string()
        .ok_or_else(|| "provider callback resolved to a non-string value".to_string())
}

/// Forward one core `ProgressEvent` to the host callback as a plain JS
/// object (`{type: "key_done", ...}`). A throwing observer does not
/// abort the run, matching `Progress::on_event`'s infallible contract.
fn emit(on_event: &js_sys::Function, event: &ProgressEvent) -> Result<(), JsError> {
    let value = to_js(event)?;
    let _ = on_event.call1(&JsValue::NULL, &value);
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FillRunSpec {
    /// Source locale file contents (JSON or YAML text).
    source: String,
    /// Existing target file contents; absent/empty fills everything.
    target: Option<String>,
    /// `json` | `yaml`; default `json` (ignored when `target` is yaml).
    target_format: Option<String>,
    source_locale: String,
    target_locale: String,
    locale_instruction: Option<String>,
    model: String,
    #[serde(default)]
    model_params: Map<String, Value>,
    batch_size: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FillRunReport {
    /// `key -> translated` map of keys the provider answered.
    filled: FlatMap,
    /// The filled target file serialized back to text.
    output: String,
    /// Terminal per-key failures (transport/parse batch errors,
    /// provider omissions).
    failures: Vec<FillFailure>,
}

/// Fill the target's missing keys through a host-provided transport,
/// emitting the core `ProgressEvent` stream to `onEvent`.
///
/// `runFill(spec, translate, onEvent): Promise<{filled, output, failures}>`
///
/// - `spec`: `{source, target?, targetFormat?, sourceLocale, targetLocale,
///   localeInstruction?, model, modelParams?, batchSize?}`
/// - `translate`: `(chatCompletionsBody) => Promise<string>` — post the
///   body and resolve `choices[0].message.content`; reject to fail the
///   batch (other batches still run, matching core `fill`).
/// - `onEvent`: `(event) => void` — receives each `ProgressEvent` as a
///   plain object (`run_started`, `batch_started`, `batch_finished`,
///   `key_done`, `locale_finished`, `run_finished`).
#[wasm_bindgen(js_name = runFill)]
pub async fn run_fill(
    spec: JsValue,
    translate: JsValue,
    on_event: JsValue,
) -> Result<JsValue, JsError> {
    let spec: FillRunSpec = from_js(spec)?;
    let translate = js_function(translate, "translate")?;
    let on_event = js_function(on_event, "onEvent")?;

    let source_value = intl_ai_formats::parse_auto(&spec.source).map_err(js_err)?;
    let source = flatten::flatten(&source_value);
    let target_value = match &spec.target {
        Some(text) if !text.trim().is_empty() => {
            intl_ai_formats::parse_auto(text).map_err(js_err)?
        }
        _ => Value::Object(Map::new()),
    };
    let target = flatten::flatten(&target_value);
    let missing: Vec<String> = source
        .keys()
        .filter(|k| !target.contains_key(*k))
        .cloned()
        .collect();

    let locale = spec.target_locale.clone();
    emit(
        &on_event,
        &ProgressEvent::RunStarted {
            pipeline: Pipeline::Fill,
            locales: vec![locale.clone()],
        },
    )?;

    let mut filled = FlatMap::new();
    let mut failures: Vec<FillFailure> = Vec::new();
    let batch_size = spec.batch_size.unwrap_or(20).max(1);
    for chunk in missing.chunks(batch_size) {
        emit(
            &on_event,
            &ProgressEvent::BatchStarted {
                locale: locale.clone(),
                keys: chunk.len(),
                attempt: 0,
            },
        )?;
        let body = to_js(&translate_body(TranslateBodyRequest {
            source_locale: spec.source_locale.clone(),
            target_locale: locale.clone(),
            entries: chunk
                .iter()
                .map(|k| Entry {
                    key: k.clone(),
                    source: source[k].clone(),
                })
                .collect(),
            glossary: BTreeMap::new(),
            locale_instruction: spec.locale_instruction.clone(),
            feedback: BTreeMap::new(),
            syntax_hint: None,
            model: spec.model.clone(),
            model_params: spec.model_params.clone(),
        }))?;
        // A transport or parse failure terminal-fails every key in the
        // batch; later batches still run (same policy as core fill).
        let result = match call_provider(&translate, &body).await {
            Ok(content) => parse_translations_map(&content)
                .map_err(|e| (ErrorType::ParseError, js_message(&JsValue::from(e)))),
            Err(message) => Err((ErrorType::Http, message)),
        };
        let translations = match result {
            Ok(map) => map,
            Err((kind, message)) => {
                emit(
                    &on_event,
                    &ProgressEvent::BatchFinished {
                        locale: locale.clone(),
                        attempt: 0,
                        answered: 0,
                        failed: chunk.len(),
                    },
                )?;
                for key in chunk {
                    failures.push(FillFailure {
                        locale: locale.clone(),
                        key: Some(key.clone()),
                        kind,
                        message: message.clone(),
                    });
                    emit(
                        &on_event,
                        &ProgressEvent::KeyDone {
                            locale: locale.clone(),
                            key: key.clone(),
                            outcome: KeyOutcome::Failed {
                                kind,
                                message: message.clone(),
                            },
                        },
                    )?;
                }
                continue;
            }
        };
        // Only keys we asked for: an over-eager provider must not leak
        // values for keys outside this batch (same filter as core).
        let answered = chunk
            .iter()
            .filter(|k| translations.contains_key(*k))
            .count();
        emit(
            &on_event,
            &ProgressEvent::BatchFinished {
                locale: locale.clone(),
                attempt: 0,
                answered,
                failed: chunk.len() - answered,
            },
        )?;
        for key in chunk {
            match translations.get(key) {
                Some(value) => {
                    filled.insert(key.clone(), value.clone());
                    emit(
                        &on_event,
                        &ProgressEvent::KeyDone {
                            locale: locale.clone(),
                            key: key.clone(),
                            outcome: KeyOutcome::Written {
                                origin: Origin::Ai,
                                regenerated_human: false,
                                unresolved: 0,
                                scores: BTreeMap::new(),
                            },
                        },
                    )?;
                }
                None => {
                    let message = "provider returned no translation for this key".to_string();
                    failures.push(FillFailure {
                        locale: locale.clone(),
                        key: Some(key.clone()),
                        kind: ErrorType::OutputTruncated,
                        message: message.clone(),
                    });
                    emit(
                        &on_event,
                        &ProgressEvent::KeyDone {
                            locale: locale.clone(),
                            key: key.clone(),
                            outcome: KeyOutcome::Failed {
                                kind: ErrorType::OutputTruncated,
                                message,
                            },
                        },
                    )?;
                }
            }
        }
    }

    let output = unflatten_impl(target_value, filled.clone(), spec.target_format.as_deref())?;
    emit(
        &on_event,
        &ProgressEvent::LocaleFinished {
            pipeline: Pipeline::Fill,
            locale,
        },
    )?;
    emit(
        &on_event,
        &ProgressEvent::RunFinished {
            pipeline: Pipeline::Fill,
            locales: 1,
            failures: failures.len(),
        },
    )?;
    to_js(&FillRunReport {
        filled,
        output,
        failures,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckRunSpec {
    /// Source locale file contents.
    source: String,
    /// Target file contents to check (e.g. `runFill`'s `output`).
    target: String,
    source_locale: String,
    target_locale: String,
    locale_instruction: Option<String>,
    /// Builtin check ids: `icu`, `placeholder-parity`, `dialect:<locale>`.
    check_ids: Vec<String>,
    /// Keys to score via the judge callback (`{key, locale, source,
    /// translation}`); empty skips the provider round trip.
    #[serde(default)]
    judge_items: Vec<JudgeEntry>,
    model: String,
    #[serde(default)]
    model_params: Map<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FindingOut {
    key: String,
    kind: FindingKind,
    check: String,
    message: String,
    cached: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckRunReport {
    findings: Vec<FindingOut>,
    judgements: Vec<JudgementOut>,
    /// Check-level errors (`{locale}/{check}: {error}`); any entry marks
    /// the run as failed in `run_finished`.
    errors: Vec<String>,
}

/// Check a target file against the source, emitting `ProgressEvent`s to
/// `onEvent` as findings land. `judge` shares `runFill`'s transport
/// shape; pass `null`/`undefined` (or leave `judgeItems` empty) to skip
/// scoring.
///
/// `runCheck(spec, judge, onEvent): Promise<{findings, judgements, errors}>`
///
/// - `spec`: `{source, target, sourceLocale, targetLocale,
///   localeInstruction?, checkIds, judgeItems?, model, modelParams?}`
/// - events: `run_started`, `finding` (per structural diff and per
///   check finding), `batch_started`/`batch_finished` around the judge
///   round trip, `locale_finished`, `run_finished`.
#[wasm_bindgen(js_name = runCheck)]
pub async fn run_check(
    spec: JsValue,
    judge: JsValue,
    on_event: JsValue,
) -> Result<JsValue, JsError> {
    let mut spec: CheckRunSpec = from_js(spec)?;
    let judge = if judge.is_null() || judge.is_undefined() {
        None
    } else {
        Some(js_function(judge, "judge")?)
    };
    let on_event = js_function(on_event, "onEvent")?;

    let source_value = intl_ai_formats::parse_auto(&spec.source).map_err(js_err)?;
    let source = flatten::flatten(&source_value);
    let target_value = intl_ai_formats::parse_auto(&spec.target).map_err(js_err)?;
    let target = flatten::flatten(&target_value);

    let locale = spec.target_locale.clone();
    emit(
        &on_event,
        &ProgressEvent::RunStarted {
            pipeline: Pipeline::Check,
            locales: vec![locale.clone()],
        },
    )?;

    let mut findings: Vec<FindingOut> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    // Structural buckets the demo can derive without a lockfile:
    // source keys the target lacks (missing) and target keys the
    // source no longer has (extra). Stale/modified/unreviewed need
    // provenance and stay native-only.
    for (kind, keys) in [
        (
            FindingKind::Missing,
            source
                .keys()
                .filter(|k| !target.contains_key(*k))
                .cloned()
                .collect::<Vec<_>>(),
        ),
        (
            FindingKind::Extra,
            target
                .keys()
                .filter(|k| !source.contains_key(*k))
                .cloned()
                .collect::<Vec<_>>(),
        ),
    ] {
        for key in keys {
            emit(
                &on_event,
                &ProgressEvent::Finding {
                    locale: locale.clone(),
                    kind,
                    key: key.clone(),
                    check: String::new(),
                    message: String::new(),
                    cached: false,
                },
            )?;
            findings.push(FindingOut {
                key,
                kind,
                check: String::new(),
                message: String::new(),
                cached: false,
            });
        }
    }

    let items: Vec<CheckItem> = target
        .iter()
        .map(|(key, value)| CheckItem {
            key: key.clone(),
            source: source.get(key).cloned(),
            target: value.clone(),
        })
        .collect();
    let ctx = CheckCtx {
        source_locale: &spec.source_locale,
        target_locale: &locale,
        transport: None,
        locale_instruction: spec.locale_instruction.as_deref(),
    };
    for id in &spec.check_ids {
        if id == "judge" {
            return Err(JsError::new(
                "judge requires a provider round trip; pass judgeItems + judge callback",
            ));
        }
        let check = builtin(id).map_err(js_err)?;
        match check.run(&ctx, &items) {
            Ok(out) => {
                for f in out.findings {
                    emit(
                        &on_event,
                        &ProgressEvent::Finding {
                            locale: locale.clone(),
                            kind: FindingKind::Invalid,
                            key: f.key.clone(),
                            check: f.check.clone(),
                            message: f.message.clone(),
                            cached: false,
                        },
                    )?;
                    findings.push(FindingOut {
                        key: f.key,
                        kind: FindingKind::Invalid,
                        check: f.check,
                        message: f.message,
                        cached: false,
                    });
                }
            }
            // A check that errors is a check-level failure, not a run
            // abort — same as core check's `report.errors`.
            Err(e) => errors.push(format!("{locale}/{id}: {e}")),
        }
    }

    let mut judgements = Vec::new();
    let judge_items = std::mem::take(&mut spec.judge_items);
    if let Some(judge_fn) = &judge
        && !judge_items.is_empty()
    {
        emit(
            &on_event,
            &ProgressEvent::BatchStarted {
                locale: locale.clone(),
                keys: judge_items.len(),
                attempt: 0,
            },
        )?;
        let body = to_js(&judge_body(JudgeBodyRequest {
            items: judge_items,
            locale_instruction: spec.locale_instruction.clone(),
            model: spec.model.clone(),
            model_params: spec.model_params.clone(),
        }))?;
        let outcome = match call_provider(judge_fn, &body).await {
            Ok(content) => {
                parse_judgements_vec(&content).map_err(|e| js_message(&JsValue::from(e)))
            }
            Err(e) => Err(e),
        };
        match outcome {
            Ok(parsed) => {
                emit(
                    &on_event,
                    &ProgressEvent::BatchFinished {
                        locale: locale.clone(),
                        attempt: 0,
                        answered: parsed.len(),
                        failed: 0,
                    },
                )?;
                judgements = parsed;
            }
            Err(e) => {
                emit(
                    &on_event,
                    &ProgressEvent::BatchFinished {
                        locale: locale.clone(),
                        attempt: 0,
                        answered: 0,
                        failed: 0,
                    },
                )?;
                errors.push(format!("{locale}/judge: {e}"));
            }
        }
    }

    emit(
        &on_event,
        &ProgressEvent::LocaleFinished {
            pipeline: Pipeline::Check,
            locale,
        },
    )?;
    emit(
        &on_event,
        &ProgressEvent::RunFinished {
            pipeline: Pipeline::Check,
            locales: 1,
            failures: errors.len(),
        },
    )?;
    to_js(&CheckRunReport {
        findings,
        judgements,
        errors,
    })
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
