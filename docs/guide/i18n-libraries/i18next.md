---
title: i18next
description: AI-translate i18next locale files at build time. Works with any AI model, zero runtime cost.
---

# i18next

This guide covers i18next consumption. For bundler setup, see [Build systems](/guide/build-systems/).

## Overview

intl-ai generates translation JSON files at build time. i18next consumes these files at runtime. This guide shows how to integrate both tools.

## Installation

::: tabs

== tab "npm"

```sh
npm install @intl-ai/unplugin i18next react-i18next
```

== tab "pnpm"

```sh
pnpm add @intl-ai/unplugin i18next react-i18next
```

== tab "yarn"

```sh
yarn add @intl-ai/unplugin i18next react-i18next
```

== tab "bun"

```sh
bun add @intl-ai/unplugin i18next react-i18next
```

:::

You only need `@intl-ai/unplugin`. It runs the `intl-ai` binary, which the `intl-ai` npm package installs automatically.

## i18next Syntax Note

i18next uses `{{variable}}` for interpolation (not ICU `{variable}`). For example:

```json
{
  "greeting": "Hello, {{name}}!",
  "items": "You have {{count}} items"
}
```

## Configuration

Create an `intl-ai.toml` at your project root. See [Configuration](/guide/configuration) for the full schema.

```toml
locale_dir = "public/locales"
source = "en"
targets = ["es", "de"]

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

Omit `processor` (the passthrough default) so i18next's `{{var}}` placeholders survive untouched.

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
