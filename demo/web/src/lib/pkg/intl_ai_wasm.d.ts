/* tslint:disable */
/* eslint-disable */

/**
 * The adversarial judge batch body (system prompt, per-item
 * source/translation block, `judgements` json_schema, temperature 0).
 * Same fetch target as `buildTranslateBody` — JS scores the filled
 * file in a second request instead of wiring a transport callback.
 *
 * `buildJudgeBody(req): object`
 */
export function buildJudgeBody(req: any): any;

/**
 * The exact OpenAI-compatible chat body the native `http` transport
 * would send for this translation batch: system+user prompts, strict
 * `json_schema` response_format, temperature 0.3, `modelParams`
 * spread last. The host JS posts it to `{baseUrl}/chat/completions`.
 *
 * `buildTranslateBody(req): object`
 */
export function buildTranslateBody(req: any): any;

/**
 * Flatten a locale file's contents (JSON or YAML text) into a flat
 * `key -> string` map.
 *
 * `flatten(sourceJson): Record<string, string>`
 */
export function flatten(source: string): any;

export function init(): void;

/**
 * Source keys with no counterpart in the target map, sorted — the set
 * `fill` would translate for a fresh or partial locale file.
 *
 * `missingKeys(sourceFlat, targetFlat | null | undefined): string[]`
 */
export function missingKeys(source_flat: any, target_flat: any): any;

/**
 * `choices[0].message.content` -> per-key adversarial scores, with the
 * same lenient payload extraction as `parseTranslations`.
 *
 * `parseJudgements(content): [{key, score, reason?, errors}]`
 */
export function parseJudgements(content: string): any;

/**
 * `choices[0].message.content` -> `key -> translated` map. Tolerates
 * upstreams that ignore `json_schema` and wrap the payload in markdown
 * fences or extra prose (fenced-block then balanced-brace extraction,
 * same as the command transport).
 *
 * `parseTranslations(content): Record<string, string>`
 */
export function parseTranslations(content: string): any;

/**
 * Check a target file against the source, emitting `ProgressEvent`s to
 * `onEvent` as findings land. `judge` shares `runFill`'s transport
 * shape; pass `null`/`undefined` (or leave `judgeItems` empty) to skip
 * scoring.
 *
 * `runCheck(spec, judge, onEvent): Promise<{findings, judgements, errors}>`
 *
 * - `spec`: `{source, target, sourceLocale, targetLocale,
 *   localeInstruction?, checkIds, judgeItems?, model, modelParams?}`
 * - events: `run_started`, `finding` (per structural diff and per
 *   check finding), `batch_started`/`batch_finished` around the judge
 *   round trip, `locale_finished`, `run_finished`.
 */
export function runCheck(spec: any, judge: any, on_event: any): Promise<any>;

/**
 * Run builtin rule checks against flattened items. Supported ids are
 * the pure builtins: `icu`, `placeholder-parity`, `dialect:<locale>`.
 * `judge` is rejected here — it needs a provider round trip, which JS
 * performs via `buildJudgeBody`/`parseJudgements`.
 *
 * `runChecks(items, checkIds, ctx): findings[]`
 */
export function runChecks(items: any, check_ids: any, ctx: any): any;

/**
 * Fill the target's missing keys through a host-provided transport,
 * emitting the core `ProgressEvent` stream to `onEvent`.
 *
 * `runFill(spec, translate, onEvent): Promise<{filled, output, failures}>`
 *
 * - `spec`: `{source, target?, targetFormat?, sourceLocale, targetLocale,
 *   localeInstruction?, model, modelParams?, batchSize?}`
 * - `translate`: `(chatCompletionsBody) => Promise<string>` — post the
 *   body and resolve `choices[0].message.content`; reject to fail the
 *   batch (other batches still run, matching core `fill`).
 * - `onEvent`: `(event) => void` — receives each `ProgressEvent` as a
 *   plain object (`run_started`, `batch_started`, `batch_finished`,
 *   `key_done`, `locale_finished`, `run_finished`).
 */
export function runFill(spec: any, translate: any, on_event: any): Promise<any>;

/**
 * Merge flat `key -> value` entries into a locale tree, starting from
 * the existing target file contents (or an empty object), and
 * serialize the filled file back out — the same `set_nested` +
 * canonical writer `fill` uses.
 *
 * `unflatten(targetJsonOrYaml | null | undefined, map, format?): string`
 */
export function unflatten(target: any, map: any, format: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly buildJudgeBody: (a: any) => [number, number, number];
    readonly buildTranslateBody: (a: any) => [number, number, number];
    readonly flatten: (a: number, b: number) => [number, number, number];
    readonly init: () => void;
    readonly missingKeys: (a: any, b: any) => [number, number, number];
    readonly parseJudgements: (a: number, b: number) => [number, number, number];
    readonly parseTranslations: (a: number, b: number) => [number, number, number];
    readonly runCheck: (a: any, b: any, c: any) => any;
    readonly runChecks: (a: any, b: any, c: any) => [number, number, number];
    readonly runFill: (a: any, b: any, c: any) => any;
    readonly unflatten: (a: any, b: any, c: any) => [number, number, number];
    readonly wasm_bindgen_f7f1c3b9929b6523___convert__closures_____invoke___js_sys_363abde2fafe0137___Function_fn_wasm_bindgen_f7f1c3b9929b6523___JsValue_____wasm_bindgen_f7f1c3b9929b6523___sys__Undefined___js_sys_363abde2fafe0137___Function_fn_wasm_bindgen_f7f1c3b9929b6523___JsValue_____wasm_bindgen_f7f1c3b9929b6523___sys__Undefined_______true_: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_f7f1c3b9929b6523___convert__closures_____invoke___wasm_bindgen_f7f1c3b9929b6523___JsValue__core_9b3796e30d99ddb7___result__Result_____wasm_bindgen_f7f1c3b9929b6523___JsError___true_: (a: number, b: number, c: any) => [number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
