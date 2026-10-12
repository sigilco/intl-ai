---
title: Vite
description: AI-translate ficheiros de locale i18n do Vite em tempo de build. Zero runtime, qualquer modelo de IA.
---

# Vite

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

```typescript
import { defineConfig } from "vite";
import IntlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [IntlAi()],
});
```

Funciona com qualquer framework baseado em Vite (Vue, React, Svelte, Solid). Associe-se a uma biblioteca i18n de [bibliotecas i18n](/pt/guide/i18n-libraries/).
