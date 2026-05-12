---
title: SDK Reference
---

# SDK Reference

API reference for `@intl-ai/core` configuration. This is the internal package that powers intl-ai's translation pipeline — you interact with it through `intl-ai.config.ts`.

## Configuration Files

Configuration is automatically discovered from these files in your project root:

- `intl-ai.config.ts` (TypeScript, recommended)
- `intl-ai.config.js` (JavaScript)
- `.intl-airc` (JSON)

## Core Options

### `defaultLocale` (required)

**Type:** `string`

The default locale code used as the source language for translations.

```typescript
defaultLocale: "en";
```

### `locales` (required)

**Type:** `string[]`

Array of all supported locale codes in your application.

```typescript
locales: ["en", "de", "es", "fr", "ja"];
```

### `localeDir` (required)

**Type:** `string`

Path to the directory containing your locale JSON files.

```typescript
localeDir: "./locales";
```

Locale files should be named after each locale (e.g., `en.json`, `de.json`) and support dot-notation for nested keys.

---

### `model` (required)

**Type:** `LanguageModel` (from Vercel AI SDK)

The AI language model to use for translations.

```typescript
import { openai } from "@ai-sdk/openai";

model: openai("gpt-4-turbo");
```

**Supported Providers:** OpenAI, Anthropic, Google, Azure OpenAI, Cohere, Mistral, Local Models.

---

### `processor` (optional)

**Type:** `IntlAiProcessor`

Custom syntax processor for validating and extracting tokens from translation strings.

```typescript
interface IntlAiProcessor {
  name: string;
  extractTokens(message: string): string[];
  validate(source: string, translated: string): ValidationResult;
  getSyntaxHint(): string;
}

interface ValidationResult {
  valid: boolean;
  errors?: string[];
}
```

**Built-in Processors:** ICU MessageFormat (`icuProcessor`), Passthrough (`passthroughProcessor`).

```typescript
import { icuProcessor } from "@intl-ai/core";

export default {
  model: openai("gpt-4-turbo"),
  defaultLocale: "en",
  locales: ["en", "de", "es"],
  localeDir: "./locales",
  processor: icuProcessor,
};
```

**ICU MessageFormat Syntax:** `{name}`, `{count, plural, one {# item} other {# items}}`

### `glossary` (optional)

**Type:** `Record<string, string>`

Domain-specific terminology that should be preserved during translation.

```typescript
glossary: {
  "React": "React",
  "TypeScript": "TypeScript",
  "intl-ai": "intl-ai",
}
```

### `maxRetries` (optional)

**Type:** `number`

**Default:** `3`

Maximum number of retry attempts for failed translation requests.

```typescript
maxRetries: 5;
```

---

## Type Definitions

```typescript
interface IntlAiConfig {
  defaultLocale: string;
  locales: string[];
  localeDir: string;
  model: LanguageModel;
  processor?: IntlAiProcessor;
  glossary?: Record<string, string>;
  maxRetries?: number;
}

interface IntlAiProcessor {
  name: string;
  extractTokens(message: string): string[];
  validate(source: string, translated: string): ValidationResult;
  getSyntaxHint(): string;
}

interface ValidationResult {
  valid: boolean;
  errors?: string[];
}
```

---

---

## Validation

Configuration is validated using Zod schemas. Invalid configurations throw errors during plugin initialization, checking that required fields are present, types are correct, and values are within valid ranges.

---

## Environment Variables

Use environment variables for API keys as shown in the [AI Model Setup](/guide/ai-model) guide.
