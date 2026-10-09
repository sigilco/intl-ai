---
title: Webpack
description: AI-translate ficheiros de locale i18n Webpack em tempo de construção. Zero runtime, qualquer modelo de IA.
---

# Webpack

## Instalação

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

## Configuração

Crie um `intl-ai.toml` na raiz do seu projeto. Consulte [Configuração](/pt/guide/configuration) para o esquema completo.

```javascript
const IntlAi = require("@intl-ai/unplugin/webpack");

module.exports = {
  plugins: [new IntlAi()],
};
```

Para o Next.js, consulte [Next.js](/pt/guide/build-systems/next-js). Combine com uma biblioteca i18n de [bibliotecas i18n](/pt/guide/i18n-libraries/).
