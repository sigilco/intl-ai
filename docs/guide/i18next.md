---
title: i18next
---

# i18next

## Overview

intl-ai generates translation JSON files at build time. i18next consumes these files at runtime. This guide shows how to integrate both tools.

## Installation

::: code-group

```sh [npm]
npm install @intl-ai/unplugin i18next react-i18next
```

```sh [pnpm]
pnpm add @intl-ai/unplugin i18next react-i18next
```

```sh [yarn]
yarn add @intl-ai/unplugin i18next react-i18next
```

```sh [bun]
bun add @intl-ai/unplugin i18next react-i18next
```

:::

You only need `@intl-ai/unplugin`. The translation engine lives in `@intl-ai/api` and is bundled automatically.

## i18next Syntax Note

i18next uses `&#123;&#123;variable&#125;&#125;` for interpolation (not ICU `&#123;variable&#125;`). For example:

```json
{
  "greeting": "Hello, {{name}}!",
  "items": "You have {{count}} items"
}
```

## Configuration

Create an `intl-ai.config.ts` (or `.json`) at your project root. See [Configuration](/guide/configuration) for the full schema. For a live Vercel AI SDK model instance:

```typescript
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";

const openai = createOpenAICompatible({
  name: "openai",
  baseURL: "https://api.openai.com/v1",
  apiKey: process.env.OPENAI_API_KEY,
});

export default {
  model: openai("your-model-name"),
  defaultLocale: "en",
  locales: ["en", "es", "de"],
  localeDir: "./public/locales",
};
```

## Bundler Integration

::: code-group

```typescript [Vite]
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin";

export default defineConfig({
  plugins: [IntlAi.vite()],
});
```

```javascript [Webpack]
const IntlAi = require("@intl-ai/unplugin");

module.exports = {
  plugins: [new IntlAi.webpack()],
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

## Example Project

See the [Next.js example](/guide/next-js) for a complete framework integration.
