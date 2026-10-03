---
title: Configuration
description: intl-ai config file reference. intl-ai.toml, .json, or .yaml, validated against a published schema.
---

# Configuration

intl-ai reads a single config file, `intl-ai.toml`, `intl-ai.json`, `intl-ai.yaml`, or `intl-ai.yml`, discovered at your project root (first hit wins). Pass `--config <path>` to pick one explicitly, or `--config -` to read TOML from stdin.

The contract is validated against a published JSON Schema served at `https://intl-ai.illo.fyi/schema/v1.json` (`intl-ai config schema` prints the same schema).

## Minimal config

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

## Required fields

### `locale_dir`

Directory containing locale files, resolved against the config file's directory. One file per locale: `${locale_dir}/${locale}.json` (or `.yaml`).

### `source`

Source locale code, e.g. `"en"`.

### `targets`

Target locale codes to fill and check.

### `[provider]`

How translations are produced. Three kinds:

- `kind = "http"`: an OpenAI-compatible chat-completions endpoint (`provider`/`model`/`api_key`/`base_url`/`model_params`). See [Providers](/guide/providers/).
- `kind = "command"`: a local CLI agent (`command`/`args`/`agent`/`prompt_via`/`cwd`/`timeout_ms`). Allowed only in the root config file, never via `extends`.
- `kind = "replay"`: a cassette JSON file of canned translations, for tests and demos.

## Optional fields

### `api_key` interpolation

Secrets are interpolated at load: `${env:VAR}` reads an environment variable, `${file:PATH}` reads a file. `config validate` prints them masked.

```toml
api_key = "${env:OPENAI_API_KEY}"
```

### `glossary`

Fixed term-to-translation pairs injected into every translate prompt.

```toml
[glossary]
React = "React"
TypeScript = "TypeScript"
```

### `locale_instructions`

Freeform style or dialect instruction per locale, sent to the model and to the judge check. Resolution order: exact locale (`en-GB`), then language subtag (`en`), then `*` as a catch-all.

```toml
[locale_instructions]
en-GB = "Use British spelling (colour, organise)."
"*" = "Keep a formal tone."
```

Changing an instruction does not retranslate existing entries. Retranslate one locale with `intl-ai fill --locale en-GB --regenerate`.

### `batch_size`

Max source entries per translate request. Default is unlimited (all keys in one batch). Reduce for models with smaller context windows or more granular per-key failures.

### `max_retries`

Transport attempts per request before the batch fails. Default `3`, capped at `10`.

### `processor`

Placeholder contract hint sent to the model. `icu` selects ICU MessageFormat; the default is passthrough.

### `format`

Locale file format minted for new files: `json` (default), `yaml`, or a `[[formats]]` name. An existing file's own extension always wins, so mixed directories work. Unknown names fail validation.

### `extends`

Compose configs: a root config can extend a base (list or single path). Extended files may not define `provider.command`, `exec` checks, or `exec` formats: data files from a dependency must not spawn programs.

## Custom formats

### `[[formats]]`

External locale formats registered alongside the builtins. See [Format plugins](/guide/format-plugins/) for the protocol and a reference implementation.

```toml
[[formats]]
name = "xml"
exec = "python3"
args = ["tools/strings_xml.py"]
extension = "xml"
extensions = ["axml"]     # optional extra claimed extensions
cwd = "tools"            # optional; defaults to the config directory
timeout_ms = 60000       # optional
max_stdout_bytes = 10485760
```

## Checks and quality gates

### `[[checks]]`

Validation checks run by `intl-ai check` and the fill gate.

```toml
[[checks]]
id = "icu"

[[checks]]
id = "placeholder-parity"

[[checks]]
id = "judge"
threshold = 0.85
weight = 2.0
```

Builtin ids: `icu`, `placeholder-parity`, `dialect:<locale>`, `judge`. A check entry can also point at a declarative YAML spec (`spec = "checks/brand.yaml"`) or an external command (`exec = "./my-check"` with `args`/`cwd`/`timeout_ms`/`max_stdout_bytes`, root config only). `threshold` is judge-only (score below it becomes an `invalid` finding, default `0.8`); `weight` feeds the `[quality]` aggregate.

### `[check]`

```toml
[check]
fail_on = ["missing", "stale", "invalid"]
cache = true
```

`fail_on` lists finding kinds that fail `intl-ai check`. `cache` toggles the incremental findings cache (`.intl-ai/check-cache.json`).

### `[fill]`

```toml
[fill]
validate = ["icu", "placeholder-parity", "judge"]
```

The shift-left gate: these checks run inside `fill` between translate and adoption. Only feedback-eligible checks qualify (`icu`, `placeholder-parity`, `judge`). Failed keys get one corrective round; unresolved keys are adopted anyway, recorded under `quality.unresolved` in the lockfile, and `check` keeps flagging them. Override per run with `--validate`/`--no-validate`.

### `[quality]`

```toml
[quality]
fail_below = 0.5
review_below = 0.8
```

Weighted per-key score over the configured checks (binary checks score 1.0/0.0, judge scores its real 0..1). Below `fail_below` is an `invalid` finding; below `review_below` marks the key unreviewed.

## Editor intellisense

Add `"$schema": "https://intl-ai.illo.fyi/schema/v1.json"` to a JSON config for autocomplete and validation.

## CI validation

```bash
intl-ai config validate   # resolves extends, masks secrets, fails on bad shape
```

See [Providers](/guide/providers/) for provider kinds and [AI model setup](/guide/ai-model/) for endpoint selection.
