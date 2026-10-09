---
title: Rspack
description: Traduce con IA los archivos de locale i18n de Rspack en tiempo de compilación. Sin runtime, cualquier modelo de IA.
---

# Rspack

## Instalación

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

## Configuración

Crea un `intl-ai.toml` en la raíz de tu proyecto. Consulta [Configuration](/es/guide/configuration) para ver el esquema completo.

```javascript
const IntlAi = require("@intl-ai/unplugin/rspack");

module.exports = {
  plugins: [new IntlAi()],
};
```

Combínalla con una biblioteca i18n de [i18n libraries](/es/guide/i18n-libraries/).
