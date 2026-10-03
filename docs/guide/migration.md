---
title: Migrating from 0.4.x
description: Upgrade path from the TypeScript 0.4.x packages to the Rust intl-ai binary and @intl-ai/* shims.
---

# Migrating from 0.4.x

intl-ai 1.x replaces the TypeScript packages with a single Rust binary. Install it from Homebrew, mise, the install script, or the `intl-ai` npm wrapper; no Node.js is required to run it. The `@intl-ai/*` packages are thin shims that locate and spawn the binary inside your build tool.

This page maps each 0.4.x concept to its 1.x equivalent.

## Packages

| 0.4.x               | 1.x                                                                  |
| ------------------- | -------------------------------------------------------------------- |
| `@intl-ai/cli`      | `intl-ai` binary (brew, mise, install.sh, or npm wrapper)            |
| `@intl-ai/api`      | Removed; the binary is the whole API surface                         |
| `@intl-ai/unplugin` | `@intl-ai/unplugin` shim (spawn the binary in `buildStart`)          |
| `@intl-ai/next`     | `@intl-ai/next` shim (`withIntlAi()` runs the binary at config eval) |

`@intl-ai/cli` and `@intl-ai/api` stay on npm, deprecated at their last 0.x versions. If you used `@intl-ai/api` programmatically from TypeScript, move to running the binary (`intl-ai fill`, `intl-ai check`) or to `kind = "command"` provider setups.

## Install

```bash
brew install sigilco/intl-ai/intl-ai
# or
mise use -g intl-ai
# or the npm wrapper, which downloads the platform binary on install
npm install -D intl-ai
```

## Configuration

0.4.x read `intl-ai.config.{ts,js,mjs}` or `.intl-ai.json` with camelCase keys. 1.x reads `intl-ai.toml`, `intl-ai.json`, `intl-ai.yaml`, or `intl-ai.yml` with snake_case keys.

### Field mapping

| 0.4.x                         | 1.x                                                       |
| ----------------------------- | --------------------------------------------------------- |
| `defaultLocale`               | `source`                                                  |
| `locales` (minus the default) | `targets`                                                 |
| `localeDir`                   | `locale_dir`                                              |
| `glossary`                    | `glossary`                                                |
| `localeInstructions`          | `locale_instructions`                                     |
| `maxRetries`                  | `max_retries`                                             |
| `batchSize`                   | `batch_size`                                              |
| `processor`                   | `processor` (`passthrough` default, `icu`)                |
| `format`                      | `format` (`json` default, `yaml`)                         |
| `provider`, `model`           | `[provider]` `provider`, `model`                          |
| `apiKey`                      | `api_key` (interpolate secrets, see below)                |
| `baseURL`                     | `base_url`                                                |
| `modelParams`                 | `model_params`                                            |
| `kind = "agent"` + `agent`    | `[provider]` `kind = "command"` + `agent`                 |
| `quality.threshold`           | `[[checks]] id = "judge"` `threshold` + `[quality]` bands |

### Example

0.4.x:

```json
{
  "defaultLocale": "en",
  "locales": ["en", "es", "fr"],
  "localeDir": "locales",
  "provider": "openai",
  "model": "gpt-4o",
  "apiKey": "sk-..."
}
```

1.x:

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
model = "gpt-4o"
api_key = "${env:OPENAI_API_KEY}"
```

::: warning
Do not paste API keys into `intl-ai.toml`. Use `${env:VAR}` or `${file:PATH}` interpolation; `config validate` prints secrets masked.
:::

## Lockfile

0.4.x wrote provenance to a single `intl-ai.lock.json`. 1.x writes sharded `intl-ai.lock.d/<locale>.toml` files. `intl-ai migrate` (planned lockfile importer) is not implemented yet, so on first run existing keys in your locale files are recorded as human-owned: they are never overwritten or re-translated. Only missing keys get AI fills. That means the practical migration is:

1. Install the binary and translate your config as above.
2. Run `intl-ai fill` once; it writes fresh `intl-ai.lock.d/` shards and fills only what is missing.
3. Delete `intl-ai.lock.json`.

If you previously relied on AI origin tracking to re-translate changed sources (`--stale` semantics), those keys are human-owned after migration; use `intl-ai fill --regenerate` or `--include-human` selectively to re-translate them once.

## Commands

| 0.4.x                  | 1.x                                              |
| ---------------------- | ------------------------------------------------ |
| `intl-ai fill`         | `intl-ai fill` (`--dry-run` previews)            |
| `intl-ai fill --force` | `intl-ai fill --regenerate` / `--include-human`  |
| `intl-ai check`        | `intl-ai check` (`--fail-on` for severity gates) |
| —                      | `intl-ai init`, `status`, `config validate`      |

## Validation and quality

The 0.4.x `quality.threshold` is now a first-class check system: `[[checks]]` entries for `icu`, `placeholder-parity`, `dialect:<locale>`, and `judge` (with per-check `threshold`/`weight`), aggregated by `[quality]` `fail_below`/`review_below` bands into pass/review/fail per key. See [Configuration](/guide/configuration/) and [Observability](/guide/observability/).

## Shims

If you wired intl-ai into a bundler, the shim packages changed shape:

- `@intl-ai/unplugin`: same name, but options now describe binary behavior (`fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `strict`, `config`, `cwd`, `bin`, `args`).
- `@intl-ai/next`: `withIntlAi(nextConfig, options)` runs the binary during config evaluation, so it works on both webpack and Turbopack.
- `@intl-ai/expo`: config plugin running the binary during `prebuild`.

All shims declare `intl-ai` as a dependency so the binary resolves inside npm-installed projects.
