---
title: Farm
description: AI-translate Farm traduz ficheiros de localização i18n em tempo de build. Zero tempo de execução, qualquer modelo de IA.
---

# Farm

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

Crie um `intl-ai.toml` na raiz do seu projeto. Veja [Configuration](/pt/guide/configuration) para o esquema completo.

```javascript
import IntlAi from "@intl-ai/unplugin/farm";

export default {
  plugins: [IntlAi()],
};
```

Combine com uma biblioteca i18n a partir de [i18n libraries](/pt/guide/i18n-libraries/).
