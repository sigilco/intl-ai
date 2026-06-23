---
"@intl-ai/api": major
"@intl-ai/cli": major
"@intl-ai/unplugin": major
"@intl-ai/next": major
---

Drop `@intl-ai/core`. The deprecated package has been removed from the monorepo and soft-deprecated on npm (`npm deprecate @intl-ai/core`). Consumers of `@intl-ai/core` should migrate to `@intl-ai/api` directly.

The supported config filenames are now only `intl-ai.config.ts` and `intl-ai.config.json`. The legacy `.js`, `.mjs`, `.cjs`, and `.intl-airc` filenames are no longer recognized.

Breaking changes:

- `@intl-ai/api` — `IntlAiConfig.model` is now strictly typed as `unknown` for users migrating from `@intl-ai/core` (where it was `any`).
- `@intl-ai/cli` — the `--config` flag now accepts `.ts` files in addition to `.json`.
- `@intl-ai/unplugin` and `@intl-ai/next` — the inlined `loadConfig()` searches only `intl-ai.config.ts` and `intl-ai.config.json`.
