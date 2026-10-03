---
title: Browser demo and wasm build
description: Run the intl-ai pipeline in the browser via the wasm build, or embed the crate in your tooling.
---

# Browser demo and wasm build

The `intl-ai-wasm` crate compiles the pipeline's pure pieces (flatten,
prompt building, response parsing, the builtin rule checks, unflatten)
to `wasm32-unknown-unknown` with `wasm-bindgen`.
The host JavaScript owns every IO operation: reading files, the fetch
loop against an OpenAI-compatible provider, retries, and batching.

## Try the demo

A static demo page lives in `demo/wasm/`.
Drop or paste a source locale file, set the source and target locales, and
the page fills the missing keys, runs the checks, and renders the filled
file with a download link.
Every wasm call runs inside `worker.mjs`, a module Web Worker, so
parse/check work never blocks the page; the fetch loop, retries, and
batching stay on the main thread.

::: tabs

== tab "wasm-bindgen-cli"

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
./demo/wasm/build.sh
```

== tab "wasm-pack"

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build crates/intl-ai-wasm --target web --out-dir ../../demo/wasm/pkg
```

:::

Then serve the directory over http (ES modules do not load from `file://`):

```sh
cd demo/wasm && python3 -m http.server 8080
```

## Studio SPA

`demo/web/` is a fuller Svelte SPA on the same crate: a collapsible
provider card (presets, `/models` pull, test button, `localStorage`
persistence), source and target locale selects with file-name
autodetect, streamed fill that lands row by row, a linear progress
indicator, and a per-key judge view with expandable reasons and check
findings.
See `demo/web/README.md` for the commands (`pnpm dev`,
`scripts/sync-wasm.sh`).

## Provider

The demo posts the request bodies the wasm build produces to
`{baseUrl}/chat/completions`, byte-identical to what the CLI's `http`
transport sends.
The default is `https://api.illo.fyi/v1` with model `illo-demo`, the
same gateway the [just-ai-illo](https://github.com/espetro/just-ai-illo)
project uses.

::: warning The demo must run on an _.illo.fyi origin for the default provider
`api.illo.fyi` only answers CORS requests from `_.illo.fyi`subdomains, and
its`access-control-allow-headers`does not include`authorization`.
A locally served page must point the base URL field at a different
OpenAI-compatible endpoint (`demo/wasm/mock-provider.mjs` ships a tiny
local mock for development).
An API key is only sent when the field is filled.
:::

## What the wasm API exposes

| Function                              | Purpose                                           |
| ------------------------------------- | ------------------------------------------------- |
| `flatten(sourceJson)`                 | Locale file text (JSON or YAML) to a flat map     |
| `missingKeys(sourceFlat, targetFlat)` | Source keys absent from the target, sorted        |
| `buildTranslateBody(req)`             | Chat-completions body for a translation batch     |
| `parseTranslations(content)`          | Response content to a `key -> translated` map     |
| `buildJudgeBody(req)`                 | Adversarial quality batch body                    |
| `parseJudgements(content)`            | Per-key scores, reasons, and error lists          |
| `runChecks(items, checkIds, ctx)`     | `icu`, `placeholder-parity`, `dialect:*` findings |
| `unflatten(targetJson, map)`          | Merge flat keys into the target and serialize it  |
| `runFill(spec, translate, onEvent)`   | Fill missing keys, streaming `ProgressEvent`s     |
| `runCheck(spec, judge, onEvent)`      | Check a target file, streaming `ProgressEvent`s   |

`judge` is deliberately not inside `runChecks`: it needs a provider round
trip, which the host performs with `buildJudgeBody`/`parseJudgements`.

## Streaming drivers

`runFill` and `runCheck` run the demo cut of the pipelines inside wasm
and forward the core `ProgressEvent` stream from `crates/intl-ai-core`
(`run_started`, `batch_started`, `batch_finished`, `key_done`, `finding`,
`locale_finished`, `run_finished`) to a host callback as plain JS objects:

```ts
runFill(
  {
    source, // source locale file text (JSON or YAML)
    target, // optional existing target file text
    sourceLocale,
    targetLocale,
    model,
    batchSize, // optional, default 20
  },
  translate, // (chatCompletionsBody) => Promise<contentString>
  onEvent, // (event: {type: string, ...}) => void
); // -> Promise<{filled, output, failures}>

runCheck(
  {
    source,
    target, // target = file text to check (e.g. fill output)
    sourceLocale,
    targetLocale,
    model,
    checkIds, // ["icu", "placeholder-parity", "dialect:<locale>"]
    judgeItems, // optional [{key, locale, source, translation}]
  },
  judge, // same transport shape as translate, or null
  onEvent,
); // -> Promise<{findings, judgements, errors}>
```

The transport callbacks keep IO host-side: the host posts the body to
`{baseUrl}/chat/completions` and resolves `choices[0].message.content`.
A rejected promise terminal-fails that batch's keys; later batches still
run, matching the CLI's per-batch failure policy.

## Differences from the CLI

The wasm build is a demo cut, not a port of `fill`:

- No lockfile shards, staleness detection, or provenance: the demo
  translates "missing keys" only.
- No config discovery, stat/check caches, or shard locks: those are
  filesystem-bound and stay native-only.
- No `spec` or `exec` checks, and no `command`/`replay`/`http`
  transports: `std::process` and the TLS stack do not exist on
  `wasm32-unknown-unknown`, so providers expose them behind cargo
  features the wasm crate leaves off.
- Retry/backoff lives in the host JavaScript, which also owns batching
  and progress.

## Lint the build in CI

`cargo check -p intl-ai-wasm --target wasm32-unknown-unknown` runs in CI
(`.github/workflows/ci.yml`).
It catches accidental filesystem or transport use in the wasm graph at
compile time rather than as a runtime trap in the browser.
