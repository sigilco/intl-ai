---
title: Webpack
description: Traduce con IA los archivos de locale i18n de Webpack en tiempo de compilación. Cero runtime, cualquier modelo de IA.
---

# Webpack

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
const IntlAi = require("@intl-ai/unplugin/webpack");

module.exports = {
  plugins: [new IntlAi()],
};
```

Para Next.js, consulta [Next.js](/es/guide/build-systems/next-js). Combínalo con una librería i18n de [i18n libraries](/es/guide/i18n-libraries/).
