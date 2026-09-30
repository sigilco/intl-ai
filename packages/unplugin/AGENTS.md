# packages/unplugin — Agent Context

Universal bundler shim via [unplugin 3](https://github.com/unjs/unplugin). A thin wrapper: all translation logic lives in the `intl-ai` Rust binary; this package only spawns it at the right bundler lifecycle point. Adapters for Vite, Rollup, Webpack, esbuild, Rspack, Rolldown, Farm, and Bun ship as subpath exports.

---

## Architecture

| File/Dir            | Purpose                                                                                 |
| ------------------- | --------------------------------------------------------------------------------------- |
| `src/index.ts`      | `unpluginFactory()` — `buildStart` resolves the binary and runs `fill` (+ `check` gate) |
| `src/run.ts`        | `IntlAiPluginOptions`, binary resolution (`resolveIntlAiBin`), argv builders, `spawn`   |
| `src/types.ts`      | Public type re-exports                                                                  |
| `src/<bundler>.ts`  | Two-line subpath adapters (`unplugin.vite`, `unplugin.webpack`, ...)                    |
| `src/index.test.ts` | Shim tests: argv mapping, buildStart against a stub executable                          |
| `src/bin.test.ts`   | Binary-resolution tests (mocks `createRequire` to reach the PATH/not-found branches)    |

---

## How It Works

```typescript
// buildStart:
const bin = resolveIntlAiBin(o.bin, cwd); // bin opt > INTL_AI_BIN > intl-ai npm pkg > PATH
await runIntlAi(bin, buildFillArgs(o), cwd); // spawn, stdio inherited into build log
if (failOn.length) await runIntlAi(bin, buildCheckArgs(o, failOn), cwd);
```

- Binary resolution prefers the `intl-ai` npm package (a runtime dependency, resolved via `createRequire` so pnpm layouts work) over PATH.
- Option mapping: `fill` -> run/skip fill; `validate` -> `--validate`/`--no-validate`; `judgeThreshold` -> `--judge-threshold`; `failOn` -> `check --fail-on`; `config` -> `--config`; `args` -> appended to `fill`; `dev: false` -> skip when `NODE_ENV !== "production"`.
- `buildStart` context in unplugin 3 has no `warn`/`error`: strict failures `throw` (fails the build), `strict: false` downgrades to `console.warn`.

---

## Adding a New Bundler Adapter

1. Check the adapter exists on the `UnpluginInstance` (`Object.keys(unplugin)` — e.g. `.bun`, `.farm`).
2. Add `src/<bundler>.ts` re-exporting `unplugin.<bundler>`, add the `./<bundler>` subpath in `package.json` exports, and add the bundler to tsdown externals + optional peer deps.

No changes needed to the factory itself — unplugin handles the rest.

---

## Test Patterns

- `index.test.ts` spawns a stub executable (`intl-ai-stub.cjs` written to tmpdir) passed via the `bin` option; it records argv to `INTL_AI_STUB_LOG` and exits per `INTL_AI_STUB_EXIT` / `INTL_AI_STUB_CHECK_EXIT`.
- `bin.test.ts` mocks `node:module`'s `createRequire` so the npm-package branch can't resolve inside this workspace; do not remove that mock or the PATH tests will hit the installed `intl-ai` dependency instead.
- Never load real bundler configs in unit tests.
