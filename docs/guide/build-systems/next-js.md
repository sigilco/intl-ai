---
title: Next.js
---

# Next.js

Next.js 15+ defaults to Turbopack. Use `@intl-ai/next` to register the Turbopack loader. For Next.js 14 with webpack only, use `@intl-ai/unplugin/webpack` directly.

## Installation

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

## Configuration

Create an `intl-ai.config.ts` at your project root. See [Configuration](/guide/configuration) for the full schema.

```typescript
import withIntlAi from "@intl-ai/next";

export default withIntlAi({
  reactStrictMode: true,
});
```

No changes to your app code required. Translations are generated at build time with zero runtime overhead.

## Next.js 14 (webpack only)

If you are on Next.js 14 with webpack, use `@intl-ai/unplugin/webpack` directly:

```typescript
import intlAiWebpackPlugin from "@intl-ai/unplugin/webpack";

export default {
  webpack(config) {
    config.plugins = config.plugins || [];
    config.plugins.push(intlAiWebpackPlugin());
    return config;
  },
};
```

Pair with an i18n library from [i18n libraries](/guide/i18n-libraries/).
