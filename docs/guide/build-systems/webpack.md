---
title: Webpack
---

# Webpack

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

```javascript
const IntlAi = require("@intl-ai/unplugin/webpack");

module.exports = {
  plugins: [new IntlAi()],
};
```

For Next.js, see [Next.js](/guide/build-systems/next-js). Pair with an i18n library from [i18n libraries](/guide/i18n-libraries/).
