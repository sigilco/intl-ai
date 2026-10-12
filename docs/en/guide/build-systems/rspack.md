---
title: Rspack
description: AI-translate Rspack i18n locale files at build time. Zero runtime, any AI model.
---

# Rspack

## Installation

::: tabs

== tab "npm"

```sh
npm install @intl-ai/unplugin
```

== tab "pnpm"

```sh
pnpm add @intl-ai/unplugin
```

== tab "yarn"

```sh
yarn add @intl-ai/unplugin
```

== tab "bun"

```sh
bun add @intl-ai/unplugin
```

:::

## Configuration

Create an `intl-ai.toml` at your project root. See [Configuration](/guide/configuration) for the full schema.

```javascript
const IntlAi = require("@intl-ai/unplugin/rspack");

module.exports = {
  plugins: [new IntlAi()],
};
```

Pair with an i18n library from [i18n libraries](/guide/i18n-libraries/).
