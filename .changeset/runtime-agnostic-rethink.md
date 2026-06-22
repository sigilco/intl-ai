---
"@intl-ai/api": major
"@intl-ai/cli": minor
"@intl-ai/unplugin": minor
"@intl-ai/next": minor
---

Runtime-agnostic rethink: reduce the public API surface, replace AJV with Zod, and consolidate documentation.

- `@intl-ai/api` now exposes only `runFill`, `IntlAiConfig`, `RunFillOptions`, `RunFillResult`, and `IntlAiConfigSchema`. All other modules live behind the `@intl-ai/api/internal` subpath export.
- `@intl-ai/cli` validates JSON configs with `IntlAiJsonConfigSchema` from `@intl-ai/api/internal`. AJV is removed.
- JSON Schema for `intl-ai.config.json` is generated from Zod and served at `/intl-ai/schema/v1.json`.
- `@intl-ai/core` is deprecated and will be removed in v1.0.
- Docs are consolidated under `/guide/` with provider-neutral AI model setup, a single installation page, and LLM-friendly output via `vitepress-plugin-llms`.
