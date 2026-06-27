---
title: Configuration
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

Directory containing locale JSON files.

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
