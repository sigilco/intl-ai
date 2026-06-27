---
title: API reference
---

# API reference

`@intl-ai/api` is the runtime-agnostic core of intl-ai. It exposes one function and one config type.

## `runFill(config, options?)`

```typescript
import { runFill } from "@intl-ai/api";
import type { IntlAiConfig, RunFillOptions, RunFillResult } from "@intl-ai/api";

const result: RunFillResult = await runFill(config, {
  locale: "es",
  force: false,
  dryRun: false,
});
```

The function walks your locale files, translates missing keys, updates the lockfile, and writes the new translations to disk.

### Options

| Option   | Type      | Description                            |
| -------- | --------- | -------------------------------------- |
| `locale` | `string`  | Translate only this locale.            |
| `force`  | `boolean` | Re-translate human-edited entries.     |
| `dryRun` | `boolean` | Preview changes without writing files. |

### Result

| Field        | Type       | Description                        |
| ------------ | ---------- | ---------------------------------- |
| `translated` | `number`   | Number of keys translated.         |
| `skipped`    | `number`   | Number of keys already up to date. |
| `errors`     | `number`   | Number of translation errors.      |
| `locales`    | `string[]` | Locales that were processed.       |

## `IntlAiConfig`

```typescript
interface IntlAiConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  model: AIProvider | string; // provider ID string or AIProvider instance
  apiKey: ApiKeyValue;        // supports $VAR and ${VAR} env interpolation
  baseURL: string;
  modelParams?: Record<string, unknown>; // passthrough to provider
  hook?: TranslationHook;
  processor?: IntlAiProcessor;
  glossary?: Record<string, string>;
  maxRetries?: number;
}
```

For JSON config files, use `IntlAiJsonConfigSchema` and `jsonConfigToIntlAiConfig` from `@intl-ai/api/internal`.

## JSON Schema

A JSON Schema for `intl-ai.config.json` is published at:

```text
https://www.schemastore.org/intl-ai.json
```

Add it to your config for editor intellisense and CI validation:

```json
{
  "$schema": "https://www.schemastore.org/intl-ai.json",
  "defaultLocale": "en",
  "locales": ["en", "es"],
  "localeDir": "./locales",
  "provider": "openai",
  "model": "gpt-4o-mini",
  "apiKey": "${OPENAI_API_KEY}",
  "baseURL": "https://api.openai.com/v1"
}
```
