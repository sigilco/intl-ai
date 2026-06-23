# @intl-ai/next

[![npm](https://img.shields.io/npm/v/@intl-ai/next?style=flat-square)](https://www.npmjs.com/package/@intl-ai/next)

Turbopack bridge for `@intl-ai/unplugin` in Next.js 15+ apps. Provides `withIntlAi()` to register the Turbopack locale loader and delegate webpack integration to `@intl-ai/unplugin/webpack`.

## Install

```bash
npm install -D @intl-ai/next
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

Then wrap your Next.js config:

```ts
// next.config.ts
import withIntlAi from "@intl-ai/next";

export default withIntlAi({
  // your existing Next.js config
  reactStrictMode: true,
});
```

No changes to your app code required. Translations are generated at build time with zero runtime overhead.

## Turbopack Support

Next.js 15+ with Turbopack is fully supported. The `withIntlAi()` wrapper registers the Turbopack loader (for `next dev --turbopack`) and delegates webpack builds to `@intl-ai/unplugin/webpack`.

### Next.js 14 (webpack only)

If you are on Next.js 14 with webpack and do not need Turbopack, you can use `@intl-ai/unplugin/webpack` directly in your `next.config.ts`:

```ts
import intlAiWebpackPlugin from "@intl-ai/unplugin/webpack";

export default {
  webpack(config) {
    config.plugins = config.plugins || [];
    config.plugins.push(intlAiWebpackPlugin());
    return config;
  },
};
```

[Documentation](https://intl-ai.pages.dev/) · [Report an issue](https://github.com/sigilco/intl-ai/issues)
