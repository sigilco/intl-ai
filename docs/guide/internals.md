---
title: Internals
description: intl-ai internals for contributors. Hexagonal architecture, package layout, and design decisions.
---

# Internals

This page is for contributors who want to understand how intl-ai is built. User-facing documentation lives under [Guide](/guide/getting-started).

## Package layout

| Package             | Purpose                                                                                                                                |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `@intl-ai/api`      | Runtime-agnostic translation core. Public surface: `runFill`, `IntlAiConfig`, `RunFillOptions`, `RunFillResult`, `IntlAiConfigSchema`. |
| `@intl-ai/cli`      | `intl-ai fill` and `intl-ai check` commands. Loads `intl-ai.config.ts` or `intl-ai.config.json`.                                       |
| `@intl-ai/unplugin` | Universal bundler plugin adapters. Loads config and calls `runFill` at `buildStart`.                                                   |
| `@intl-ai/next`     | Next.js wrapper around the webpack plugin and Turbopack loader. Loads config at startup and on the webpack `emit` hook.                |

## Internal subpath

Modules that SDK consumers do not need are exposed through `@intl-ai/api/internal`. Sibling packages in this monorepo may import them, but external SDKs and plugins should stay on the public surface. The internal barrel includes engine, lockfile, processor, formats, utilities, and JSON config schemas.

## JSON Schema generation

`packages/api/src/schema/intl-ai.schema.json` is generated from `IntlAiJsonConfigSchema` in `packages/api/src/schema/json-config.ts`. Run:

```bash
pnpm --filter @intl-ai/api schema:build
```

This writes both the package-local schema and `docs/public/schema/v1.json`, which GitHub Pages serves at `/intl-ai/schema/v1.json`.

## SchemaStore

The schema is submitted to SchemaStore so editors discover it automatically for files matching `intl-ai.config.ts` and `intl-ai.config.json`. See the plan at `.agents/plans/2026-06-22-runtime-agnostic-rethink.md` for the catalog entry format.

## Release pipeline

- Binaries are built with `bun build --compile` for `bun-darwin-arm64`, `bun-linux-x64`, and `bun-linux-arm64`.
- npm packages are published with changesets.
- Docs are built with VitePress and deployed to GitHub Pages.

See `.github/workflows/release.yml` for details.
