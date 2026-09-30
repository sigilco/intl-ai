---
title: Internals
description: How intl-ai is structured. Rust core crates, the binary, and the npm shims.
---

# Internals

## Workspace layout

The translation engine is a Rust workspace; the npm packages are thin shims that spawn the binary.

| Package                                   | Purpose                                                                                                                                  |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/intl-ai-core`                     | Config loading (`intl-ai.toml`/`.json`/`.yaml`), interpolation, lockfile shards (`intl-ai.lock.d/`), provenance.                         |
| `crates/intl-ai-formats`                  | Locale file formats: JSON and YAML, flatten/unflatten, stat cache.                                                                       |
| `crates/intl-ai-providers`                | Transports: `http` (OpenAI-compatible), `command` (local agents), `replay` (cassettes). Prompt contract + retries.                       |
| `crates/intl-ai-checks`                   | Validation framework: `icu`, `placeholder-parity`, `dialect`, `judge`, declarative specs, `exec` checks, the incremental findings cache. |
| `crates/intl-ai-cli`                      | The `intl-ai` binary: `fill`, `check`, `mark`, `review`, `status`, `lockfile`, `config`, `migrate`.                                      |
| `packages/unplugin` (`@intl-ai/unplugin`) | Bundler shim (vite, webpack, rollup, esbuild, rspack, rolldown, farm, bun). Runs the pipeline in `buildStart`.                           |
| `packages/next` (`@intl-ai/next`)         | `withIntlAi()` config wrapper. Runs the pipeline during `next.config` evaluation, before webpack or Turbopack.                           |
| `packages/expo` (`@intl-ai/expo`)         | Expo config plugin. Runs the pipeline during `expo prebuild`/`eas build`.                                                                |
| `intl-ai` (npm)                           | Binary delivery: postinstall downloads the platform tarball from GitHub Releases. The shims resolve it via `createRequire`.              |

## Release pipeline

`cargo dist` generates the release workflow: tag `vX.Y.Z` produces platform tarballs, `intl-ai-installer.sh`/`.ps1`, a homebrew formula, npm wrapper publish, and GitHub attestations. See [Versioning in AGENTS.md](https://github.com/sigilco/intl-ai/blob/develop/AGENTS.md#versioning).

## Config schema

The config contract is generated from the Rust types and committed at `docs/public/schema/intl-ai.schema.json`; `docs/public/schema/v1.json` is the published copy. CI fails on drift, so edit the types and regenerate with `intl-ai config schema`.
