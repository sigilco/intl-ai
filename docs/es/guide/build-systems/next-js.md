---
title: Next.js
description: Traducción con IA en tiempo de compilación para i18n de Next.js. Sin sobrecarga en tiempo de ejecución, cualquier modelo de IA.
---

# Next.js

`@intl-ai/next` envuelve tu `next.config` para que el binario `intl-ai` ejecute `fill` antes de que comience la compilación. Funciona con ambos empaquetadores de Next.js, webpack y Turbopack; consulta la [documentación de Next.js sobre Turbopack](https://nextjs.org/docs/app/api-reference/config/next-config-js/turbopack) para saber cuál usa tu versión. Versión mínima compatible: Next.js 14+.

## Instalación

::: tabs

== tab "npm"

```sh
npm install @intl-ai/next
```

== tab "pnpm"

```sh
pnpm add @intl-ai/next
```

== tab "yarn"

```sh
yarn add @intl-ai/next
```

== tab "bun"

```sh
bun add @intl-ai/next
```

:::

## Configuración

Crea un `intl-ai.toml` en la raíz de tu proyecto. Consulta [Configuración](/es/guide/configuration) para ver el esquema completo.

```typescript
import { withIntlAi } from "@intl-ai/next";

export default withIntlAi({
  reactStrictMode: true,
});
```

No se requieren cambios en el código de tu aplicación. `intl-ai fill` se ejecuta mientras Next.js evalúa la configuración, antes de que el empaquetador comience, por lo que las traducciones se obtienen en tiempo de compilación sin sobrecarga en tiempo de ejecución.

Las opciones de traducción van en el segundo argumento y reflejan las opciones de [`@intl-ai/unplugin`](/es/guide/build-systems/):

```typescript
export default withIntlAi(
  { reactStrictMode: true },
  {
    failOn: ["missing", "invalid"], // build fails on these findings
    dev: false, // skip during `next dev`
  },
);
```

## Alternativa: adaptador de unplugin

Si prefieres el hook del empaquetador en lugar del envoltorio de configuración (o no quieres la dependencia), usa `@intl-ai/unplugin/webpack` directamente en tu callback de webpack:

```typescript
import intlAiWebpackPlugin from "@intl-ai/unplugin/webpack";

export default {
  webpack(config) {
    config.plugins = config.plugins || [];
    config.plugins.push(intlAiWebpackPlugin());
    return config;
  },
};
```

Combínalo con una biblioteca de i18n de [bibliotecas i18n](/es/guide/i18n-libraries/).
