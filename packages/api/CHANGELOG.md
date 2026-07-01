# @intl-ai/api

## 0.4.0

### Minor Changes

- 49d7109: feat(fill): quality-aware fill loop with LLM-as-a-Judge retry. `runFill(config, { quality })` now runs an optional quality loop: after each fill batch, the same provider judges the translations, entries below the configured threshold are refilled with the judge's feedback, and a build plugin enabled with `quality: true` fails the build when keys remain below threshold after retries. Threshold and `maxRetries` come from `intl-ai.config.json`; the build plugin option only toggles the loop. Custom judges plug in via `config.quality.assessor: QualityAssessorInstance`. Closes #14.

### Patch Changes

- fix(config): thread JSON config `model` field to `translateBatch`/`judgeBatch`. The `model` field in `intl-ai.config.json` was being ignored in favour of the hardcoded `gpt-4o-mini` fallback. `model` is now a required top-level string in the JSON schema, mapped to `config.model` and passed all the way through to the batch translators. Lockfile now records the actual model name used.
- fix(config): accept the JSON Schema meta-key `$schema` in the JSON config parser. Editors rely on `$schema` for autocomplete against `https://www.schemastore.org/intl-ai.json`; rejected alongside model-thread fix in v0.4.0.

## 0.3.0

### Minor Changes

- v0.3.0: refactor docs, narrow Next.js integration, and add alternative install paths.
  - Docs restructured with central `<DocsUrl>` URL management, a new `Build systems` section (Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, Farm, Bun, Next.js), and an `i18n libraries` compatibility page.
  - `@intl-ai/next` narrowed to a Next.js 15+ Turbopack bridge. Webpack builds now delegate to `@intl-ai/unplugin/webpack`, removing duplicated config loading and eager `runFill()` startup.
  - Added Homebrew tap (`brew install sigilco/tap/intl-ai`) and mise registry (`mise use npm:intl-ai`) support.
  - All documentation links now point to `https://intl-ai.pages.dev` instead of the previous `illo.fyi` domain.

  The v0.2.0 tag was created prematurely and is replaced by v0.3.0.
