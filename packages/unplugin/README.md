# @intl-ai/unplugin

[![npm](https://img.shields.io/npm/v/@intl-ai/unplugin?style=flat-square)](https://www.npmjs.com/package/@intl-ai/unplugin)

AI-powered i18n translation plugin for all bundlers.

## Install

```bash
npm install -D @intl-ai/unplugin
```

## Quick Start

Create an `intl-ai.config.ts` (or `.json`) at your project root:

```ts
// intl-ai.config.ts
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";

const openai = createOpenAICompatible({
  name: "openai",
  baseURL: "https://api.openai.com/v1",
  apiKey: process.env.OPENAI_API_KEY,
});

export default {
  model: openai("your-model-name"),
  defaultLocale: "en",
  locales: ["en", "es", "fr"],
  localeDir: "./locales",
};
```

Then add the plugin to your bundler config:

### Vite

```ts
// vite.config.ts
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [
    IntlAi({
      sourceLanguage: "en",
      targetLanguages: ["es", "fr", "de"],
    }),
  ],
});
```

### Webpack

```js
// webpack.config.js
const IntlAi = require("@intl-ai/unplugin/webpack").default;

module.exports = {
  plugins: [new IntlAi({ sourceLanguage: "en", targetLanguages: ["es", "fr", "de"] })],
};
```

### Rollup

```js
// rollup.config.js
import IntlAi from "@intl-ai/unplugin/rollup";

export default {
  plugins: [IntlAi({ sourceLanguage: "en", targetLanguages: ["es", "fr", "de"] })],
};
```

### esbuild

```js
// esbuild.config.js
import { esbuildPlugin } from "@intl-ai/unplugin/esbuild";

export default {
  plugins: [esbuildPlugin({ sourceLanguage: "en", targetLanguages: ["es", "fr", "de"] })],
};
```

### Rspack

```js
// rspack.config.js
const IntlAi = require("@intl-ai/unplugin/rspack").default;

module.exports = {
  plugins: [new IntlAi({ sourceLanguage: "en", targetLanguages: ["es", "fr", "de"] })],
};
```

### Bun

```ts
// bunfig.toml
[plugins][plugins.prebuild];
script = "intl-ai fill";
```

Or use the plugin directly:

```ts
import IntlAi from "@intl-ai/unplugin/bun";

export default {
  plugins: [IntlAi({ sourceLanguage: "en", targetLanguages: ["es", "fr", "de"] })],
};
```

### Nuxt

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@intl-ai/unplugin/nuxt"],
  intlAi: {
    sourceLanguage: "en",
    targetLanguages: ["es", "fr", "de"],
  },
});
```

Zero-config usage also works — the plugin auto-discovers `intl-ai.config.{ts,js,intl-airc}` from your project root.

[Documentation](https://intl-ai.illo.fyi/) · [Report an issue](https://github.com/espetro/intl-ai/issues)
