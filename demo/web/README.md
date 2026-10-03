# intl-ai studio

Svelte SPA demo for `crates/intl-ai-wasm`: drop or paste a source locale
file, pick an OpenAI-compatible provider, and watch the fill, check, and
judge pipeline run in the browser. All wasm work happens in a Web Worker
so the page never blocks.

## Stack

- Svelte 5 + Vite 8 + Tailwind 4 + daisyUI (custom `illodark` theme)
- Vercel AI SDK (`ai` + `@ai-sdk/openai-compatible`) for the provider
  calls; zod schemas mirror `crates/intl-ai-providers/src/chat.rs`
- `crates/intl-ai-wasm` bundle served from `src/lib/pkg` inside a module
  worker (`src/worker.ts`)

## Develop

```sh
pnpm install
./scripts/sync-wasm.sh   # build + copy the wasm bundle (needs wasm-bindgen-cli)
pnpm dev
```

The demo needs a CORS-friendly OpenAI-compatible endpoint. For local work
either pick the **Custom** preset and point Base URL at the mock:

```sh
node ../wasm/mock-provider.mjs   # http://localhost:8787/v1
```

or use any endpoint that answers cross-origin (OpenRouter preset works
with an API key). The illo preset targets `https://api.illo.fyi`, which
only answers `*.illo.fyi` origins.

Provider config (preset, base URL, key, model, batch size, stream/judge
toggles) persists to `localStorage` across reloads.

## Notes

- `stream fill` uses `streamObject` and updates rows as partial objects
  arrive; it falls back to a plain request when the endpoint has no SSE.
- `quality judge` sends a second structured request after fill; per-row
  chips show the score and expand to reason/errors plus check findings.
- Source locale `auto` detects from the dropped file name (e.g.
  `en.json`, `locales/de.json`); explicit select always wins.
- GenerationView (the per-line judge viewer) is the one custom component;
  everything else is daisyUI stock.
