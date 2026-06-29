---
title: Getting Started
description: Set up intl-ai in minutes. Install the plugin, configure your AI model, and translate.
---

# Getting Started

This guide will help you set up `@intl-ai/unplugin` in your project.

## Installation

### Prerequisites

- Node.js 22+
- A package manager: npm, pnpm, or yarn
- An AI model provider (see [AI Model Setup](/guide/ai-model) for options)

### Install the Package

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

```sh [bun]
bun add @intl-ai/next
```

:::

## Quick Start

### 1. Create Configuration File

Create an `intl-ai.config.ts` file in your project root.
If you do not have a local model or cloud API key, you can use OpenRouter's free tier with no account setup beyond an API key.

### Local model (LM Studio)

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

export default {
  provider: resolveProvider("openai"),
  model: "qwen3.5-4b-instruct",
  apiKey: "lm-studio",
  baseURL: "http://127.0.0.1:1234/v1",
  defaultLocale: "en",
  locales: ["en", "de", "es", "fr"],
  localeDir: "./locales",
};
```

### Cloud model (OpenRouter free tier)

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

const openrouter = resolveProvider("openai");

export default {
  provider: openrouter,
  model: "google/gemini-2.0-flash-exp:free",
  apiKey: "${OPENROUTER_API_KEY}",
  baseURL: "https://openrouter.ai/api/v1",
  defaultLocale: "en",
  locales: ["en", "de", "es", "fr"],
  localeDir: "./locales",
};
```

See [AI model setup](/guide/ai-model) for all provider options.

**Key Configuration:**

- `provider`: Provider ID or AIProvider instance (e.g. `"openai"`, `"anthropic"`, or a custom provider)
- `model`: Model name passed to the provider
- `apiKey`: Your API key (use `${ENV_VAR}` for environment variables)
- `baseURL`: Provider endpoint URL
- `defaultLocale`: The primary language for your application

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
const IntlAiPlugin = require("@intl-ai/unplugin/webpack");

module.exports = {
  plugins: [new IntlAiPlugin()],
};
```

:::

See [Build systems](/guide/build-systems/) for Next.js, Rollup, esbuild, Rspack, Rolldown, Farm, and more.

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

### 4. Run Translation

Run the CLI to fill in translations for your target locales:

```bash
intl-ai fill
```

Or use `runFill` programmatically from `@intl-ai/api`:

```typescript
import { runFill } from "@intl-ai/api";
import type { IntlAiConfig } from "@intl-ai/api";

const config: IntlAiConfig = {
  defaultLocale: "en",
  locales: ["en", "de", "es", "fr"],
  localeDir: "./locales",
  model: "openai",
  apiKey: "${OPENAI_API_KEY}",
  baseURL: "https://api.openai.com/v1",
};

const result = await runFill(config);
// { locales: ["de", "es", "fr"], translated: 12, skipped: 0, errors: 0 }
```

This produces `locales/de.json`, `locales/es.json`, and `locales/fr.json` with AI-generated translations, plus a `locales/intl-ai.lock.json` lockfile that tracks which keys were translated and their source hashes.

### 5. Optional: Check Translation Quality

After filling, run `check` to detect missing keys, stale translations (keys whose source changed), and extra keys with no source:

```bash
intl-ai check
```

Or use `runCheck` programmatically from `@intl-ai/api`:

```typescript
import { runCheck } from "@intl-ai/api";

const result = await runCheck(config, { locale: "es" });
// { results: [{ locale: "es", missing: [...], stale: [...], extra: [...] }], hasIssues: true }
```

`runCheck` is read-only. It writes nothing to disk and does not call hooks (hooks fire only during `runFill`). Use it in CI to enforce translation completeness before deploying.

## Supported Bundlers

`@intl-ai/unplugin` works with all major bundlers. See [Build systems](/guide/build-systems/) for dedicated setup guides:

- [Vite](/guide/build-systems/vite) - Modern, fast build tool
- [Webpack](/guide/build-systems/webpack) - Industry standard bundler
- [Rollup](/guide/build-systems/rollup) - Flexible module bundler
- [esbuild](/guide/build-systems/esbuild) - Extremely fast JavaScript bundler
- [Rspack](/guide/build-systems/rspack) - Rust-based, webpack-compatible bundler
- [Rolldown](/guide/build-systems/rolldown) - Rust-powered Rollup-compatible bundler
- [Farm](/guide/build-systems/farm) - Rust-based web build tool
- [Next.js](/guide/build-systems/next-js) - React framework with Turbopack bridge

## Verify Installation

To verify everything is working:

1. Start your development server: `npm run dev`, `pnpm dev`, or `yarn dev`
2. Check that your bundler loads without errors and translation files are being processed
3. Verify translations render correctly in your application

If you encounter issues, check the [AI model setup](/guide/ai-model) guide to ensure your model provider is configured correctly.
