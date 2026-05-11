---
title: Getting Started
---

# Getting Started

This guide will help you set up `unplugin-intl-ai` in your project.

## Installation

### Prerequisites

- Node.js 22+
- A package manager: npm, pnpm, or yarn
- An AI model provider (see [AI Model Setup](/guide/ai-model) for options)

### Install the Package

::: code-group

```sh [npm]
npm install unplugin-intl-ai
```

```sh [pnpm]
pnpm add unplugin-intl-ai
```

```sh [yarn]
yarn add unplugin-intl-ai
```

```sh [JSR]
npx jsr add @intl-ai/unplugin
```

:::

### For Next.js Projects

Install the `@intl-ai/next` package in your Next.js project:

::: code-group

```sh [npm]
npm install @intl-ai/next
```

```sh [pnpm]
pnpm add @intl-ai/next
```

```sh [yarn]
yarn add @intl-ai/next
```

:::

## Quick Start

### 1. Create Configuration File

Create an `intl-ai.config.ts` file in your project root:

```typescript
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";

const lmstudio = createOpenAICompatible({
  name: "lmstudio",
  baseURL: "http://127.0.0.1:1234/v1",
});

export default {
  model: lmstudio("qwen3.5-4b-instruct"),
  defaultLocale: "en",
  locales: ["en", "de", "es", "fr"],
  localeDir: "./locales",
};
```

**Key Configuration:**

- `model`: Your AI model instance (see [AI Model Setup](/guide/ai-model) for other providers)
- `defaultLocale`: The primary language for your application
- `locales`: Array of supported language codes
- `localeDir`: Directory where translation files will be stored

### 2. Set Up Your Bundler

::: code-group

```typescript [Vite]
import { defineConfig } from "vite";
import intlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [intlAi()],
});
```

```javascript [Webpack]
const IntlAiPlugin = require("unplugin-intl-ai/webpack");

module.exports = {
  plugins: [new IntlAiPlugin()],
};
```

```javascript [Rollup]
import IntlAiPlugin from "unplugin-intl-ai/rollup";

export default {
  plugins: [IntlAiPlugin()],
};
```

```javascript [esbuild]
import { build } from "esbuild";
import IntlAiPlugin from "unplugin-intl-ai/esbuild";

build({
  plugins: [IntlAiPlugin()],
});
```

```javascript [Rspack]
const IntlAiPlugin = require("unplugin-intl-ai/rspack");

module.exports = {
  plugins: [new IntlAiPlugin()],
};
```

```javascript [Next.js]
const withIntlAi = require("@intl-ai/next");

module.exports = withIntlAi({
  // Your other Next.js config options
});
```

:::

### 3. Create Directory and Translation Files

Create the directory specified in your config (default: `./locales`), then add your first translation file for the default locale:

**locales/en.json:**

```json
{
  "greeting": "Hello, {name}!",
  "welcome": "Welcome to our application",
  "description": "This is a sample translation"
}
```

## Supported Bundlers

`unplugin-intl-ai` works with all major bundlers:

- **Vite** - Modern, fast build tool
- **Webpack** - Industry standard bundler
- **Rollup** - Flexible module bundler
- **esbuild** - Extremely fast JavaScript bundler
- **Rspack** - Rust-based bundler (Webpack-compatible)
- **Next.js** - React framework with Turbopack support

## Verify Installation

To verify everything is working:

1. Start your development server: `npm run dev`, `pnpm dev`, or `yarn dev`
2. Check that your bundler loads without errors and translation files are being processed
3. Verify translations render correctly in your application

If you encounter issues, check the [AI Model Setup](/guide/ai-model) guide to ensure your model provider is configured correctly.