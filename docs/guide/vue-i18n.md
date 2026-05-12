---
title: Vue (vue-i18n)
---

# Vue (vue-i18n)

## Overview

intl-ai generates translation JSON files at build time. vue-i18n consumes these files at runtime in your Vue application. This guide shows how to set up both tools together.

## Installation

Install intl-ai and vue-i18n:

::: code-group

```sh [npm]
npm install @intl-ai/core @intl-ai/unplugin vue-i18n
```

```sh [pnpm]
pnpm add @intl-ai/core @intl-ai/unplugin vue-i18n
```

```sh [yarn]
yarn add @intl-ai/core @intl-ai/unplugin vue-i18n
```

:::

> **Note:** `@intl-ai/core` is a dependency of `@intl-ai/unplugin` and will be installed automatically. Install it explicitly if you need direct imports from `@intl-ai/core` (e.g., `defineConfig`, `icuProcessor`).

## Configuration

## Configuration

### intl-ai.config.ts

Create an `intl-ai.config.ts` file in your project root:

```typescript
import { defineConfig } from "@intl-ai/core";
import { icuProcessor } from "@intl-ai/core";

export default defineConfig({
  defaultLocale: "en",
  locales: ["en", "es", "fr"],
  localeDir: "./locales",
  processor: icuProcessor, // Enable ICU MessageFormat validation
  model: /* your AI model */,
});
```

### Vite Integration

In your `vite.config.ts`:

```typescript
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import IntlAi from "@intl-ai/unplugin";

export default defineConfig({
  plugins: [
    vue(),
    IntlAi.vite({
      configPath: "./intl-ai.config.ts",
    }),
  ],
});
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

vue-i18n supports ICU MessageFormat via the `@formatjs/icu-messageformat-parser` package. The `icuProcessor` ensures AI-generated translations preserve ICU placeholders correctly.

## Example Project

See the [Vite example](/guide/getting-started.html) for a complete working setup.
