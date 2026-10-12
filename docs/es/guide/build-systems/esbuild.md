---
title: esbuild
description: Genera archivos de localización de i18n esbuild en tiempo de compilación. Sin ejecución en tiempo de ejecución, cualquier modelo de IA.
---

# esbuild

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

Crea un `intl-ai.toml` en la raíz de tu proyecto. Ve [Configuration](/es/guide/configuration) para el esquema completo.

```javascript
import IntlAi from "@intl-ai/unplugin/esbuild";
import { build } from "esbuild";

build({
  plugins: [IntlAi()],
});
```

Empareja con una librería i18n desde [i18n libraries](/es/guide/i18n-libraries/).
