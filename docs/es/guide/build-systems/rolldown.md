---
title: Rolldown
description: Traduce con IA los archivos de localización i18n de Rolldown en tiempo de compilación. Sin ejecución en tiempo de ejecución, cualquier modelo de IA.
---

# Rolldown

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

Crea un `intl-ai.toml` en la raíz de tu proyecto. Consulta [Configuration](/es/guide/configuration) para el esquema completo.

```javascript
import IntlAi from "@intl-ai/unplugin/rolldown";

export default {
  plugins: [IntlAi()],
};
```

Empareja con una biblioteca i18n de [i18n libraries](/es/guide/i18n-libraries/).
