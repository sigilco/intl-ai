---
title: Configuration
description: intl-ai config file reference. Single JSON or TypeScript file, validated against a published schema.
---

# Configuration

intl-ai reads a single config file. The JSON format is validated against a published JSON Schema and works in any runtime.

## Config file discovery

The CLI and bundler plugins look for one of these files in your project root:

- `intl-ai.config.json` (recommended for runtime-agnostic setups and non-Node consumers)
- `intl-ai.config.ts` (when you need a custom AIProvider instance)

## JSON config

```json
{
  "$schema": "https://www.schemastore.org/intl-ai.json",
  "defaultLocale": "en",
  "locales": ["en", "es", "fr"],
  "localeDir": "./locales",
  "provider": "openai",
  "model": "gpt-4o-mini",
  "apiKey": "${OPENAI_API_KEY}",
  "baseURL": "https://api.openai.com/v1",
  "maxRetries": 3
}
```

## Required options

### `defaultLocale`

Source language for translations.

```json
"defaultLocale": "en"
```

### `locales`

All supported locale codes.

```json
"locales": ["en", "es", "fr"]
```

### `localeDir`

Directory containing locale files. The structure depends on the chosen `format`:

- JSON (default): `${localeDir}/${locale}.json`
- YAML: `${localeDir}/${locale}.yaml`

```json
"localeDir": "./locales"
```

### `provider`

Provider ID. Built-in providers: `openai`, `anthropic`. Custom providers can pass an `AIProvider` instance directly.

```json
"provider": "openai"
```

For a custom provider, use `resolveProvider` from `@intl-ai/api/internal`:

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

export default {
  provider: resolveProvider("openai"),
  model: "gpt-4o-mini",
  apiKey: "${OPENAI_API_KEY}",
  defaultLocale: "en",
  locales: ["en", "es"],
  localeDir: "./locales",
};
```

### `apiKey`

API key for your provider. We recommend reading it from an environment variable.

```json
"apiKey": "${OPENAI_API_KEY}"
```

### `model`

Model name passed to the provider. The model name format depends on your provider.

```json
"model": "gpt-4o-mini"
```

## Optional options

### `baseURL`

Provider endpoint. Defaults to `https://api.openai.com/v1`.

```json
"baseURL": "https://api.openai.com/v1"
```

### `glossary`

Terms to preserve during translation.

```json
"glossary": {
  "React": "React",
  "TypeScript": "TypeScript"
}
```

### `maxRetries`

Maximum retry attempts for failed translations. Default is `3`.

```json
"maxRetries": 3
```

### `processor`

Syntax processor. Use `icu` for ICU MessageFormat or omit for passthrough.

```json
"processor": "icu"
```

Built-in processors: `passthrough`, `icu`.

## Locale formats

intl-ai supports JSON and YAML locale files out of the box.

### JSON (default)

JSON is the default format. No configuration needed:

```json
{
  "greeting": "Hello",
  "farewell": "Goodbye"
}
```

### YAML

YAML supports nested keys and is useful for larger projects:

```yaml
greeting: Hello
farewell: Goodbye
nested:
  welcome: Welcome back
```

To use YAML, set `format` in your config:

```json
{
  "format": "yaml"
}
```

### Other formats

For custom locale formats (CSV, TOML, or custom file formats), intl-ai does not include a built-in adapter. Use one of these approaches:

**Interactive translation (recommended for most cases):** Use the `intl-ai-translate-fill` skill in an opencode agent session. The skill is format-agnostic and works with any file format.

**Batch CI translation:** Build a custom `LocaleFormat` adapter using the `@intl-ai/api` package. See `intl-ai-format-strategy` for guidance on when to build an adapter vs. use a skill.

**Rule of thumb:** Start with the skill. Build an adapter only when you need batch CI on a format the CLI does not support natively.

### Batch size

`batchSize` controls how many keys are sent in a single translation request. Default is unlimited (all keys in one batch). Reduce for models with lower context windows or to get more granular per-key quality control.

```json
{
  "batchSize": 50
}
```

For most JSON and YAML files, the default (unlimited batch) works well.

## Editor intellisense

Add `"$schema": "https://www.schemastore.org/intl-ai.json"` to your JSON config for autocomplete and validation in VS Code, JetBrains, and other editors.

## CI validation

Validate a config file in CI with any JSON Schema tool:

```bash
# check-jsonschema
pip install check-jsonschema
check-jsonschema --schemafile https://www.schemastore.org/intl-ai.json intl-ai.config.json
```

## TypeScript config

When you need to pass a custom AIProvider instance, use a TypeScript config:

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

export default {
  provider: resolveProvider("openai"),
  model: "gpt-4o-mini",
  apiKey: "${OPENAI_API_KEY}",
  baseURL: "https://api.openai.com/v1",
  defaultLocale: "en",
  locales: ["en", "es"],
  localeDir: "./locales",
};
```

See [Providers](/guide/providers) for how the provider system works, and [AI model setup](/guide/ai-model) for provider options.

### Parallel locale processing (CLI)

When using the CLI (`intl-ai fill`), `--concurrency` controls how many locales are processed in parallel. Default is 4.

```bash
intl-ai fill --concurrency 8
```

Range: 1 to 16.
