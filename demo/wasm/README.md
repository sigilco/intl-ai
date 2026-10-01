# intl-ai wasm demo

A static page that runs the intl-ai pipeline in the browser: drop or paste a
source locale file, set the source and target locales, and the wasm build fills
the missing keys through an OpenAI-compatible provider, then runs the builtin
checks (`icu`, `placeholder-parity`, `dialect:*`) on the result. All wasm calls
run inside `worker.mjs` (a module Web Worker) so the main thread stays
responsive; fetch and batching stay on the page.

## Build

Requires Rust with the `wasm32-unknown-unknown` target and `wasm-bindgen-cli`
matching the lockfile's `wasm-bindgen` version.

```sh
./build.sh
```

This produces `pkg/` (gitignored) next to `index.html`.

## Serve

The page uses ES modules, so it must be served over http:

```sh
cd demo/wasm
python3 -m http.server 8080
```

## Provider

The default provider is `https://api.illo.fyi/v1` with model `illo-demo`.
Its CORS policy only answers origins under `*.illo.fyi`, so a locally served
page must point the base URL at a different OpenAI-compatible endpoint (the
"API key" field is sent as `Authorization: Bearer` only when filled).

## Scope

This is a demo cut of the pipeline, not the full `fill` command: no lockfile
shards, no staleness detection, no config discovery. See
`/docs/guide/wasm/` for the API surface the `intl-ai-wasm` crate exposes.
