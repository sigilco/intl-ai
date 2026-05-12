---
title: i18next
---

# i18next

## Overview

intl-ai generates translation JSON files at build time. i18next consumes these files at runtime. This guide shows how to integrate both tools, including a custom processor for i18next's `{{variable}}` interpolation syntax.

## Installation

::: code-group

```sh [npm]
npm install @intl-ai/core @intl-ai/unplugin i18next react-i18next
```

```sh [pnpm]
pnpm add @intl-ai/core @intl-ai/unplugin i18next react-i18next
```

```sh [yarn]
yarn add @intl-ai/core @intl-ai/unplugin i18next react-i18next
```

:::

> **Note:** `@intl-ai/core` is a dependency of `@intl-ai/unplugin` and will be installed automatically. Install it explicitly if you need direct imports from `@intl-ai/core` (e.g., `createProcessor`).

## i18next Syntax Note

i18next uses `{{variable}}` for interpolation (not ICU `{variable}`). For example:

```json
{
  "greeting": "Hello, {{name}}!",
  "items": "You have {{count}} items"
}
```

## Custom Processor

Create a custom processor using `createProcessor()` from `@intl-ai/core`:

```typescript
import { createProcessor } from "@intl-ai/core";

const i18nextProcessor = createProcessor({
  name: "i18next",
  extractTokens: (message: string) => {
    const matches = message.match(/\{\{(\w+)\}\}/g);
    return matches ? matches.map((m) => m.replace(/\{\{|\}\}/g, "")) : [];
  },
  validate: (source: string, translated: string) => {
    const extract = (msg: string) => {
      const matches = msg.match(/\{\{(\w+)\}\}/g);
      return matches ? matches.map((m) => m.replace(/\{\{|\}\}/g, "")) : [];
    };
    const sourceTokens = extract(source);
    const translatedTokens = extract(translated);
    const missing = sourceTokens.filter((t) => !translatedTokens.includes(t));
    const extra = translatedTokens.filter((t) => !sourceTokens.includes(t));
    if (missing.length > 0 || extra.length > 0) {
      return {
        valid: false,
        errors: [
          ...(missing.length > 0 ? [`Missing tokens: ${missing.join(", ")}`] : []),
          ...(extra.length > 0 ? [`Extra tokens: ${extra.join(", ")}`] : []),
        ],
      };
    }
    return { valid: true };
  },
  getSyntaxHint: () =>
    'i18next: Use {{variable}} for placeholders, e.g., "Hello {{name}}". Never use {variable} or other syntax.',
});
```

## Configuration

### intl-ai.config.ts

```typescript
import { defineConfig } from "@intl-ai/core";
import { i18nextProcessor } from "./i18next-processor";

export default defineConfig({
  defaultLocale: "en",
  locales: ["en", "es", "de"],
  localeDir: "./public/locales",
  processor: i18nextProcessor,
  model: /* your AI model */,
});
```

### Bundler Integration

::: code-group

```typescript [Vite]
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin";

export default defineConfig({
  plugins: [
    IntlAi.vite({
      configPath: "./intl-ai.config.ts",
    }),
  ],
});
```

```javascript [Webpack]
const IntlAi = require("@intl-ai/unplugin");

module.exports = {
  plugins: [
    IntlAi.webpack({
      configPath: "./intl-ai.config.ts",
    }),
  ],
};
```

:::

## React App Usage

```typescript
import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en.json";
import es from "./locales/es.json";

i18next.use(initReactI18next).init({
  lng: "en",
  fallbackLng: "en",
  resources: {
    en: { translation: en },
    es: { translation: es },
  },
});
```

### Using Translations

```tsx
import { useTranslation } from "react-i18next";

function App() {
  const { t } = useTranslation();
  return (
    <div>
      <p>{t("greeting", { name: "World" })}</p>
      <p>{t("items", { count: 5 })}</p>
    </div>
  );
}
```

## Processor Benefits

The custom processor ensures:

- AI translations preserve `{{variable}}` placeholders
- Missing or malformed placeholders are caught at build time
- The LLM receives syntax-specific instructions via `getSyntaxHint()`

## Example Project

See the [Next.js example](/guide/next-js.html) for a complete framework integration.
