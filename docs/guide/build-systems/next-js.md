---
title: Next.js
description: Build-time AI translation for Next.js i18n. Zero runtime overhead, any AI model.
---

# Next.js

`@intl-ai/next` wraps your `next.config` so the `intl-ai` binary runs `fill` before the build starts. It works with both Next.js bundlers, webpack and Turbopack; see the [Next.js docs on Turbopack](https://nextjs.org/docs/app/api-reference/config/next-config-js/turbopack) for which one your version uses. Minimum supported version: Next.js 14+.

## Installation

::: tabs

== tab "npm"

```sh
npm install @intl-ai/next
```

== tab "pnpm"

```sh
pnpm add @intl-ai/next
```

== tab "yarn"

```sh
yarn add @intl-ai/next
```

== tab "bun"

```sh
bun add @intl-ai/next
```

:::

## Configuration

Create an `intl-ai.toml` at your project root. See [Configuration](/guide/configuration) for the full schema.

```typescript
import { withIntlAi } from "@intl-ai/next";

export default withIntlAi({
  reactStrictMode: true,
});
```

No changes to your app code required. `intl-ai fill` runs while Next.js evaluates the config, before the bundler starts, so translations land at build time with zero runtime overhead.

Translation options go in the second argument and mirror [`@intl-ai/unplugin`](/guide/build-systems/) options:

```typescript
export default withIntlAi(
  { reactStrictMode: true },
  {
    failOn: ["missing", "invalid"], // build fails on these findings
    dev: false, // skip during `next dev`
  },
);
```

## Alternative: unplugin adapter

If you prefer the bundler hook over the config wrapper (or do not want the dependency), use `@intl-ai/unplugin/webpack` directly in your webpack callback:

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
