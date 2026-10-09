---
title: Vue (vue-i18n)
description: Traduce con IA los archivos de locale de vue-i18n en tiempo de compilación. Funciona con cualquier modelo de IA, sin costo en tiempo de ejecución.
---

# Vue (vue-i18n)

Esta guía cubre el consumo de vue-i18n. Para la configuración del empaquetador, consulta [Build systems](/es/guide/build-systems/).

## Resumen

intl-ai genera archivos JSON de traducción en tiempo de compilación. vue-i18n consume estos archivos en tiempo de ejecución en tu aplicación Vue. Esta guía muestra cómo configurar ambas herramientas juntas.

## Instalación

Instalar intl-ai y vue-i18n:

::: tabs

== tab "npm"

```sh
npm install @intl-ai/unplugin vue-i18n
```

== tab "pnpm"

```sh
pnpm add @intl-ai/unplugin vue-i18n
```

== tab "yarn"

```sh
yarn add @intl-ai/unplugin vue-i18n
```

== tab "bun"

```sh
bun add @intl-ai/unplugin vue-i18n
```

:::

Solo necesitas `@intl-ai/unplugin`. Ejecuta el binario `intl-ai`, que el paquete npm `intl-ai` instala automáticamente.

## Configuración

Crea un `intl-ai.toml` en la raíz de tu proyecto. Consulta [Configuration](/es/guide/configuration) para ver el esquema completo.

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]
processor = "icu"

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

## Configuración de la aplicación Vue

### Estructura de archivos de locale

intl-ai genera archivos JSON como:

```json
// locales/en.json
{
  "greeting": "Hello, {name}!",
  "items": {
    "one": "You have {count} item",
    "other": "You have {count} items"
  }
}
```

### Crear instancia i18n

```typescript
import { createI18n } from "vue-i18n";
import en from "./locales/en.json";
import es from "./locales/es.json";

const i18n = createI18n({
  locale: "en",
  fallbackLocale: "en",
  messages: { en, es },
});

app.use(i18n);
```

### Usar traducciones

En plantillas de Vue:

```vue
<template>
  <p>{{ $t("greeting", { name: "World" }) }}</p>
  <p>{{ $t("items.one", { count: 5 }) }}</p>
</template>
```

En Composition API:

```vue
<script setup>
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>
```

## Nota del procesador

vue-i18n admite ICU MessageFormat. Establece `processor = "icu"` en `intl-ai.toml` para que las traducciones generadas por IA preserven correctamente los marcadores de posición de ICU.
