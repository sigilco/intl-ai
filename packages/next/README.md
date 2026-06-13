# @intl-ai/next

[![npm](https://img.shields.io/npm/v/@intl-ai/next?style=flat-square)](https://www.npmjs.com/package/@intl-ai/next)

AI-powered i18n translation plugin for Next.js.

## Install

```bash
npm install -D @intl-ai/next
```

## Quick Start

Create an `intl-ai.config.{ts,js,intl-airc}` at your project root, then wrap your Next.js config:

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
  reactStrictMode: true,
});
```

No changes to your app code required — translations are generated at build time with zero runtime overhead.

## Turbopack Support

Next.js 15+ with Turbopack is fully supported. The same `withIntlAi()` wrapper registers both the Webpack plugin (for `next build`) and the Turbopack loader (for `next dev` with Turbopack).

[Documentation](https://intl-ai.illo.fyi/) · [Report an issue](https://github.com/espetro/intl-ai/issues)
