# packages/expo — Agent Context

Expo config plugin for the `intl-ai` binary. `withIntlAi` registers a `withDangerousMod` on ios and android so `intl-ai fill` (+ the `check --fail-on` gate) runs while Expo evaluates config plugins during `expo prebuild`/`eas build`/`expo run:*`. A thin shim: all translation logic lives in the binary.

---

## Architecture

| File/Dir            | Purpose                                                                  |
| ------------------- | ------------------------------------------------------------------------ |
| `src/index.ts`      | `withIntlAi` via `createRunOncePlugin`; dangerous mod with run-once flag |
| `src/index.test.ts` | Shim tests against a stub `intl-ai` executable (records argv)            |
| `app.plugin.js`     | Expo plugin entry — `require`s the cjs dist build                        |

The spawn/binary-resolution logic is shared via `@intl-ai/unplugin/runner` (`runIntlAiPipeline`, `IntlAiPluginOptions`); do not duplicate it here. `/runner` is deliberately unplugin-free so the CJS `app.plugin.js` path never loads the ESM-only `unplugin` dependency.

---

## How It Works

- One run-once flag covers both platform mods so `expo prebuild` (which evaluates ios and android) runs the pipeline exactly once.
- `cwd` defaults to `modRequest.projectRoot` (the app root), so `intl-ai.toml` discovery and `locale_dir` resolve relative to the app.
- `dev: false` skips when `NODE_ENV !== "production"` (same semantic as the bundler shims).
- Packaging: `app.plugin.js` requires `dist/index.cjs`; tsdown emits both esm and cjs, and `main` points at the cjs build for Expo's `require`-based plugin resolution.

---

## Test Patterns

- Same stub-binary pattern as `@intl-ai/unplugin`/`@intl-ai/next`: `intl-ai-stub.cjs` in a tmpdir, argv recorded to `INTL_AI_STUB_LOG`, exit codes via env.
- Drive `config.mods.<platform>.dangerous(fakeModConfig)` directly; `fakeModConfig` only needs `modRequest.projectRoot`. Never run a real `expo prebuild` in unit tests.
