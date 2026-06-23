---
title: i18next
---

# i18next

This guide covers i18next consumption. For bundler setup, see [Build systems](/guide/build-systems/).

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

i18next uses `{{variable}}` for interpolation (not ICU `{variable}`). For example:

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
