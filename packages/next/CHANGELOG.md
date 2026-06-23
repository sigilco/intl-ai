# @intl-ai/next

## 0.3.0

### Minor Changes

- v0.3.0: refactor docs, narrow Next.js integration, and add alternative install paths.

  - Docs restructured with central `<DocsUrl>` URL management, a new `Build systems` section (Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, Farm, Bun, Next.js), and an `i18n libraries` compatibility page.
  - `@intl-ai/next` narrowed to a Next.js 15+ Turbopack bridge. Webpack builds now delegate to `@intl-ai/unplugin/webpack`, removing duplicated config loading and eager `runFill()` startup.
  - Added Homebrew tap (`brew install sigilco/tap/intl-ai`) and mise registry (`mise use npm:intl-ai`) support.
  - All documentation links now point to `https://intl-ai.pages.dev` instead of the previous `illo.fyi` domain.

  The v0.2.0 tag was created prematurely and is replaced by v0.3.0.

### Patch Changes

- Updated dependencies
  - @intl-ai/api@0.3.0
  - @intl-ai/unplugin@0.3.0

## 0.2.0

### Minor Changes

- 448b5b4: v0.2.0 release: `@intl-ai/unplugin` and `@intl-ai/next` join `@intl-ai/api` and `@intl-ai/cli` at v0.2.0.

  This release also includes the runtime-agnostic rethink (Zod replacing AJV, trimmed public API surface, config file consolidation) and the config loader cleanup (core removed, only .ts/.json config files supported) which brought `@intl-ai/api` and `@intl-ai/cli` to v0.2.0.
