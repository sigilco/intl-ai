---
title: Rspack
description: AI-translate ficheiros de localização Rspack em tempo de build. Zero runtime, qualquer modelo AI.
---

# Rspack

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

Crie um `intl-ai.toml` na sua raiz de projeto. Veja [Configuration](/pt/guide/configuration) para o esquema completo.

```javascript
const IntlAi = require("@intl-ai/unplugin/rspack");

module.exports = {
  plugins: [new IntlAi()],
};
```

Emparelhe-se com uma biblioteca i18n a partir de [i18n libraries](/pt/guide/i18n-libraries/).
