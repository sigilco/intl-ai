---
title: Vite
---

# Vite

## Installation

::: code-group

```sh [npm]
npm install @intl-ai/unplugin
```

```sh [pnpm]
pnpm add @intl-ai/unplugin
```

```sh [yarn]
yarn add @intl-ai/unplugin
```

```sh [bun]
bun add @intl-ai/unplugin
```

:::

## Configuration

Create an `intl-ai.config.ts` at your project root. See [Configuration](/guide/configuration) for the full schema.

```typescript
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [IntlAi()],
});
```

Works with any Vite-based framework (Vue, React, Svelte, Solid). Pair with an i18n library from [i18n libraries](/guide/i18n-libraries).
