---
title: Farm
description: Traduce con IA los archivos de localización i18n de Farm en tiempo de compilación. Sin tiempo de ejecución, cualquier modelo de IA.
---

# Farm

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

Crea un `intl-ai.toml` en la raíz de tu proyecto. Consulta [Configuración](/es/guide/configuration) para ver el esquema completo.

```javascript
import IntlAi from "@intl-ai/unplugin/farm";

export default {
  plugins: [IntlAi()],
};
```

Combina con una biblioteca i18n desde [bibliotecas i18n](/es/guide/i18n-libraries/).
