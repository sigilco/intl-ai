---
title: i18next
description: AI-translate traduce los archivos de locale de i18next en tiempo de compilación. Funciona con cualquier modelo de IA, sin costo de tiempo de ejecución.
---

# i18next

Esta guía cubre el consumo de i18next. Para la configuración del bundler, consulte [Build systems](/es/guide/build-systems/).

## Resumen

intl-ai genera archivos de traducción JSON en tiempo de compilación. i18next consume estos archivos en tiempo de ejecución. Esta guía muestra cómo integrar ambas herramientas.

## Instalación

::: tabs

== tab "npm"

```sh
npm install @intl-ai/unplugin i18next react-i18next
```

== tab "pnpm"

```sh
pnpm add @intl-ai/unplugin i18next react-i18next
```

== tab "yarn"

```sh
yarn add @intl-ai/unplugin i18next react-i18next
```

== tab "bun"

```sh
bun add @intl-ai/unplugin i18next react-i18next
```

:::

Solo necesitas `@intl-ai/unplugin`. Ejecuta el binario `intl-ai`, que el paquete npm `intl-ai` instala automáticamente.

## Nota de sintaxis de i18next

i18next usa `{{variable}}` para la interpolación (no ICU `{variable}`). Por ejemplo:

```json
{
  "greeting": "Hello, {{name}}!",
  "items": "You have {{count}} items"
}
```

## Configuración

Crear un `intl-ai.toml` en la raíz de tu proyecto. Ver [Configuration](/es/guide/configuration) para el esquema completo.

```toml
locale_dir = "public/locales"
source = "en"
targets = ["es", "de"]

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

Omitir `processor` (el predeterminado passthrough) para que los marcadores de posición `{{var}}` de i18next permanezcan sin cambios.

## Uso de la aplicación React

```typescript
import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en.json";
import es from "./locales/es.json";

i18next.use(initReactI18next).init({
  lng: "en",
  fallbackLng: "en",
  resources: {
    en: { translation: en },
    es: { translation: es },
  },
});
```

### Uso de traducciones

```tsx
import { useTranslation } from "react-i18next";

function App() {
  const { t } = useTranslation();
  return (
    <div>
      <p>{t("greeting", { name: "World" })}</p>
      <p>{t("items", { count: 5 })}</p>
    </div>
  );
}
```
