---
title: esbuild
description: Traduz com IA os ficheiros de locale i18n do esbuild na hora da construção. Zero em tempo de execução, qualquer modelo de IA.
---

# esbuild

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
import IntlAi from "@intl-ai/unplugin/esbuild";
import { build } from "esbuild";

build({
  plugins: [IntlAi()],
});
```

Integre com uma biblioteca i18n de [bibliotecas i18n](/pt/guide/i18n-libraries/).
