---
title: Rollup
description: Traduce con IA archivos de locale i18n de Rollup en tiempo de compilación. Cero runtime, cualquier modelo de IA.
---

# Rollup

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
import IntlAi from "@intl-ai/unplugin/rollup";

export default {
  plugins: [IntlAi()],
};
```

Combina con una biblioteca i18n de [bibliotecas i18n](/es/guide/i18n-libraries/).
