# examples — Agent Context

Reference consumer applications. Minimal context — these are not under active development and have no tests.

---

## What's Here

Each example app is a working integration test for a bundler setup:

| Example       | Bundler                   | Integration                                 |
| ------------- | ------------------------- | ------------------------------------------- |
| `next`        | Next.js (App Router)      | `withIntlAi()` wrapper on webpack/Turbopack |
| `legacy-next` | Next.js 12 (Pages Router) | `withIntlAi()` on legacy webpack            |
| `vite`        | Vite                      | `@intl-ai/unplugin` with Vite adapter       |
| `webpack`     | Webpack 5                 | `@intl-ai/unplugin` with Webpack adapter    |

---

## Setup

Each example has:

- `intl-ai.config.ts` — configuration for translations
- `locales/` — locale files (JSON or YAML)
- `package.json` — local dev script

To test against local unpublished changes:

```bash
pnpm publish:local  # starts Verdaccio at localhost:4873
cd examples/vite
pnpm install  # pulls from local Verdaccio
pnpm dev
```

---

## No Testing

Examples are reference implementations, not test suites. If you need to verify bundler integration, use the example app with `pnpm publish:local` + manual verification.

---

## Lean policy

- One folder per framework / bundler / runtime. No nested "plugin AND app" subfolders; the plugin glue lives inside the example.
- No tests inside examples. Unit tests live in `packages/*` and run via `pnpm test`.
- No CI for examples. Integration is verified manually with `pnpm publish:local`.
- One `intl-ai.config.json` (JSON, not TS) per example, aligned with the lean API surface. TS configs are still allowed for projects that need a live AI SDK `model` instance, but should include `/** @see https://www.schemastore.org/intl-ai.json */` at the top.
