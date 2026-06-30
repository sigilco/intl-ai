# @intl-ai/api

[![npm](https://img.shields.io/npm/v/@intl-ai/api?style=flat-square)](https://www.npmjs.com/package/@intl-ai/api)

Runtime-agnostic translation core for [intl-ai](https://intl-ai.pages.dev). Provides `runFill`, `runCheck`, `IntlAiConfig`, and the JSON schema. Zero Node.js-only dependencies in the domain layer.

Most users should install `@intl-ai/unplugin` (bundler plugin) or `@intl-ai/cli` instead.
Use `@intl-ai/api` directly when you need to call the translation engine programmatically or build a custom integration.

## Install

```bash
npm install @intl-ai/api
```

## Usage

```ts
import { runFill } from "@intl-ai/api";

await runFill({
  defaultLocale: "en",
  locales: ["en", "es", "fr"],
  localeDir: "./locales",
  model: "openai/gpt-4o-mini",
  apiKey: process.env.OPENAI_API_KEY,
});
```

## Config schema

```ts
import type { IntlAiConfig } from "@intl-ai/api";
```

The JSON Schema is also published and registered on SchemaStore:

```json
{
  "$schema": "https://www.schemastore.org/intl-ai.json"
}
```

## Documentation

Full configuration reference and framework guides at [intl-ai.pages.dev](https://intl-ai.pages.dev).

[Report an issue](https://github.com/sigilco/intl-ai/issues)
