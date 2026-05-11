# intl-ai

> AI-powered i18n translation plugin for all bundlers

[![GitHub stars](https://img.shields.io/github/stars/espetro/intl-ai?style=social)](https://github.com/espetro/intl-ai)

[Documentation](https://intl-ai.dev) · [Report an issue](https://github.com/espetro/intl-ai/issues)

Automatically translate your app with AI. intl-ai hooks into your build pipeline to generate and update translations on every build.

## Install

**Vite / Rollup / Webpack / esbuild / Rspack**

```bash
npm install -D unplugin-intl-ai
```

**Next.js**

```bash
npm install -D @intl-ai/next
```

## Usage

### Vite

```ts
// vite.config.ts
import { defineConfig } from "vite";
import IntlAi from "unplugin-intl-ai/vite";

export default defineConfig({
  plugins: [
    IntlAi({
      sourceLanguage: "en",
      targetLanguages: ["es", "fr", "de"],
      apiKey: process.env.INTL_AI_API_KEY,
    }),
  ],
});
```

### Next.js

```ts
// next.config.ts
import createIntlAi from "@intl-ai/next";

const withIntlAi = createIntlAi({
  sourceLanguage: "en",
  targetLanguages: ["es", "fr", "de"],
  apiKey: process.env.INTL_AI_API_KEY,
});

export default withIntlAi({
  // your existing Next.js config
});
```

## Packages

| Package                                                              | Description              | Registry  |
| -------------------------------------------------------------------- | ------------------------ | --------- |
| [`unplugin-intl-ai`](https://www.npmjs.com/package/unplugin-intl-ai) | Universal bundler plugin | npm       |
| [`@intl-ai/next`](https://www.npmjs.com/package/@intl-ai/next)       | Next.js integration      | npm + JSR |

## License

[MIT](LICENSE)
