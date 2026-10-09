---
title: Vite
description: Traduce con IA los archivos de localización de i18n de Vite en tiempo de compilación. Sin tiempo de ejecución, cualquier modelo de IA.
---

# Vite

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

```typescript
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [IntlAi()],
});
```

Funciona con cualquier framework basado en Vite (Vue, React, Svelte, Solid). Combínalo con una biblioteca de i18n de [bibliotecas i18n](/es/guide/i18n-libraries/).
