---
title: Vue (vue-i18n)
description: AI-translate vue-i18n locale files at build time. Works with any AI model, zero runtime cost.
---

# Vue (vue-i18n)

This guide covers vue-i18n consumption. For bundler setup, see [Build systems](/guide/build-systems/).

## Overview

intl-ai generates translation JSON files at build time. vue-i18n consumes these files at runtime in your Vue application. This guide shows how to set up both tools together.

## Installation

Install intl-ai and vue-i18n:

::: code-group

```sh [npm]
npm install @intl-ai/unplugin vue-i18n
```

```sh [pnpm]
pnpm add @intl-ai/unplugin vue-i18n
```

```sh [yarn]
yarn add @intl-ai/unplugin vue-i18n
```

```sh [bun]
bun add @intl-ai/unplugin vue-i18n
```

:::

You only need `@intl-ai/unplugin`. The translation engine lives in `@intl-ai/api` and is bundled automatically.

## Configuration

Create an `intl-ai.config.ts` (or `.json`) at your project root. See [Configuration](/guide/configuration) for the full schema. For a custom AIProvider instance:

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

export default {
  provider: resolveProvider("openai"),
  model: "gpt-4o-mini",
  apiKey: "${OPENAI_API_KEY}",
  baseURL: "https://api.openai.com/v1",
  defaultLocale: "en",
  locales: ["en", "es", "fr"],
  localeDir: "./locales",
  processor: "icu",
};
```

## Vue App Setup

### Locale File Structure

intl-ai generates JSON files like:

```json
// locales/en.json
{
  "greeting": "Hello, {name}!",
  "items": {
    "one": "You have {count} item",
    "other": "You have {count} items"
  }
}
```

### Create i18n Instance

```typescript
import { createI18n } from "vue-i18n";
import en from "./locales/en.json";
import es from "./locales/es.json";

const i18n = createI18n({
  locale: "en",
  fallbackLocale: "en",
  messages: { en, es },
});

app.use(i18n);
```

### Using Translations

In Vue templates:

```vue
<template>
  <p>{{ $t("greeting", { name: "World" }) }}</p>
  <p>{{ $t("items.one", { count: 5 }) }}</p>
</template>
```

In Composition API:

```vue
<script setup>
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>
```

## Processor Note

vue-i18n supports ICU MessageFormat. Set `processor: "icu"` in your config so AI-generated translations preserve ICU placeholders correctly.
