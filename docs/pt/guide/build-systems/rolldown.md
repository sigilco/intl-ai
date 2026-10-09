---
title: Rolldown
description: Traduzir com IA os ficheiros de localização i18n do Rolldown no momento da compilação. Zero tempo de execução, qualquer modelo de IA.
---

# Rolldown

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
import IntlAi from "@intl-ai/unplugin/rolldown";

export default {
  plugins: [IntlAi()],
};
```

Associe-se a uma biblioteca i18n a partir de [i18n libraries](/pt/guide/i18n-libraries/).
