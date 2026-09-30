---
title: Vite
description: AI-translate Vite i18n locale files at build time. Zero runtime, any AI model.
---

# Vite

## Installation

::: tabs

== tab "npm"

```sh
npm install @intl-ai/unplugin
```

== tab "pnpm"

```sh
pnpm add @intl-ai/unplugin
```

== tab "yarn"

```sh
yarn add @intl-ai/unplugin
```

== tab "bun"

```sh
bun add @intl-ai/unplugin
```

:::

## Configuration

Create an `intl-ai.toml` at your project root. See [Configuration](/guide/configuration) for the full schema.

```typescript
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [IntlAi()],
});
```

Works with any Vite-based framework (Vue, React, Svelte, Solid). Pair with an i18n library from [i18n libraries](/guide/i18n-libraries/).
