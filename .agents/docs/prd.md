# Product requirements document: intl-ai

## Table of contents

- [Vision](#vision)
- [Product positioning](#product-positioning)
- [Delivered](#delivered)
- [Roadmap](#roadmap)
- [Test strategy](#test-strategy)
- [Research findings](#research-findings)
- [Success metrics](#success-metrics)
- [Scope boundaries](#scope-boundaries)

## Vision

### Core objective

AI-powered build-time i18n translation plugin that works with any bundler and any AI model. Plug in, translate, ship.

### Product positioning

**Swiss army knife (modular):** core library plus CLI plus management features, not a full platform.

**Target users:** indie developers and mid-size companies.

**Core differentiators:**

1. Build-time AI translation: zero runtime overhead, works with existing i18n libraries.
2. Universal bundler support: one plugin via unplugin, works with every major bundler.
3. Model agnostic and privacy-first: use any AI model (local, self-hosted, or cloud) with no vendor lock-in.

## Delivered

All four packages are at version 0.3.0 and published to npm.

- **Core API** (`@intl-ai/api`): runtime-agnostic translation pipeline with Hexagonal architecture. Public surface is `runFill`, `runCheck`, `IntlAiConfig`, and `IntlAiConfigSchema`. Internal surface (`internal.ts`) exposes adapters and infrastructure for monorepo consumers.
- **Bundler plugin** (`@intl-ai/unplugin`): universal plugin via unplugin 3. Supports Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, and Farm, plus Bun and Nuxt adapters.
- **Next.js integration** (`@intl-ai/next`): `withIntlAi()` wrapper covering both webpack plugin and Turbopack loader paths.
- **CLI** (`@intl-ai/cli`): `intl-ai fill` and `intl-ai check` commands for use outside a bundler context (mobile, desktop, CI scripts).
- **Provider abstraction**: `adapters/providers/` with OpenAI and Anthropic adapters plus a registry for custom providers. Documented in `docs/guide/providers.md`.
- **Observability hooks** (`TranslationHook` port): `onRequest`, `onSuccess`, and `onError` callbacks wired through the fill pipeline. Documented in `docs/guide/observability.md`.

**Monetization:** free and open source only. No paid tiers, community-driven.

## Roadmap

These are the next planned increments, in priority order.

### Incremental/delta translation

Translate only new and changed keys instead of the full locale on every build. Driven by a git-diff comparison at the key level (new keys plus changed source values). Deleted key handling is deferred. See `.agents/plans/` for the delta work plan.

### Translation quality scoring

LLM-as-a-Judge: after each fill, the same provider scores each translated key 0 to 1 in a separate session. Keys below a confidence threshold are flagged in the lockfile with `quality: low`. See `.agents/plans/2026-06-26-quality-estimation-infrastructure.md`.

## Test strategy

Tests-after with vitest. Each package has colocated `*.test.ts` files. There is no E2E or browser test suite. Manual and exploratory QA of the docs site uses agent-browser (not Playwright).

## Open questions

- Delta translation: confirmed key-level git diff. Deleted key behaviour is TBD.
- Quality scoring: confidence threshold and prompt design are TBD.
- Paraglide: generic JSON support, no specific integration planned.

## Research findings

### Validated pain points

1. **Hardcoded strings in AI-generated codebases.** Teams increasingly ship apps with hundreds or thousands of hardcoded UI strings, then hit a wall when international users arrive.
2. **Translation file corruption by AI.** LLMs routinely break placeholders, rename keys, or produce invalid JSON/ICU when used naively for i18n files.
3. **Subscription fatigue for translation SaaS.** Indie developers and small teams see enterprise TMS tools (Lokalise, Crowdin, Phrase) as overpriced and over-engineered for monthly string updates.
4. **Runtime i18n overhead.** Loading translations at runtime causes layout shifts, SEO issues, and bundle bloat; developers want build-time, localized bundles.
5. **Vendor lock-in and privacy concerns.** Strong demand for bring-your-own-key models, local inference (Ollama), and open-source tooling.

### Competitive landscape

- **Lingo.dev** is the closest competitor: open-source CLI for JSON/YAML/Markdown/CSV/PO, GitHub Action, REST API, and alpha React Compiler (Next.js and Vite+React only; YC-backed).
- **Prismy** competes via GitHub-native, PR-based AI localization.
- **better-i18n, wuchale, localizejson.lovable.app** are indie tools solving pieces of the problem.
- **Paraglide JS (inlang)** is a compile-time, tree-shakeable i18n alternative with its own CLI/tooling.
- **LinguiJS, Tolgee, Weblate, Translation.io** are existing open-source options with overlapping audiences.

### The wedge

intl-ai's defensible position is the combination of:

1. Build-time translation with zero runtime overhead.
2. Framework-agnostic, universal bundler support via unplugin.
3. Works with existing i18n libraries instead of replacing them.
4. Model-agnostic AI via Vercel AI SDK (BYOK, local models supported).
5. No cloud dashboard, no vendor lock-in.

No competitor in the research matches all five simultaneously.

### What intl-ai is not

- Not a translation management platform or TMS replacement.
- Not a runtime translation library.
- Not an i18n architecture generation tool (string extraction from components is deferred).
- Not a paid SaaS.

### PMF signal

Demand is inferred but strong: i18n pain is well-documented, build-time AI translation is gaining mindshare (Lingo.dev reached 89 HN points and 215 Product Hunt upvotes), and the OSS/BYOK model resonates with indie developers. Risk is medium because Lingo.dev has first-mover advantage and stronger distribution.

## Monorepo and docs structure

**Monorepo:** pnpm workspaces plus Turborepo plus changesets. Four published packages: `@intl-ai/api`, `@intl-ai/unplugin`, `@intl-ai/next`, `@intl-ai/cli`.

**Docs guide tree** (VitePress site at `docs/`):

- `getting-started`, `installation`, `ai-model`, `configuration`, `api`, `internals`, `contributing`, `observability`, `providers`
- `build-systems/`: index, vite, webpack, rollup, esbuild, rspack, rolldown, farm, next-js
- `i18n-libraries/`: index, vue-i18n, i18next
- `mobile/`: expo, flutter, swiftui, jetpack
- `desktop/`: dotnet

## Success metrics

### Primary: adoption and community growth

- Repos using intl-ai.
- Blog posts and tutorials.
- Word of mouth in i18n and bundler communities.

### Not primarily

- Stars and downloads (vanity metrics).
- Feature milestones.

## Scope boundaries

### Include

- Build-time translation (bundler plugin).
- On-demand CLI translation (`intl-ai fill`, `intl-ai check`).
- Incremental/delta translation (git diff based).
- Translation quality scoring (LLM-as-a-Judge, same provider, different session).
- i18n library support: vue-i18n, i18next, next-intl, FormatJS, Paraglide, any JSON-based setup.
- Model agnosticism: any AI model via AI SDK compatible API.
- Documentation: one guide per supported library, one guide per AI provider.
- Privacy: no telemetry, no data collection.

### Exclude

- No cloud dashboard or SaaS platform.
- No paid tiers or license tiers.
- No user accounts or authentication.
- No translation memory hosted by the project (local git-based TM only via lockfile).
- No automatic translation updates without human consent (human in the loop).
- No CLI subcommands beyond `fill` and `check` (no serve, deploy, config wizard).
- No external evaluation service for quality scoring (same provider, different session only).
- No support for binary i18n formats (PO, XLIFF) unless community-contributed.
- No RTL-specific handling (handled by the user's i18n library).
