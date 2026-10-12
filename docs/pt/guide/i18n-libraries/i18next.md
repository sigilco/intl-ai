---
title: i18next
description: Tradução com IA dos ficheiros de localização do i18next no momento da compilação. Funciona com qualquer modelo de IA, custo zero em tempo de execução.
---

# i18next

Este guia aborda o consumo do i18next. Para a configuração do empacotador, consulte [Build systems](/pt/guide/build-systems/).

## Visão geral

intl-ai gera ficheiros JSON de tradução no momento da compilação. i18next consome estes ficheiros em tempo de execução. Este guia mostra como integrar ambas as ferramentas.

## Instalação

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

Você só precisa de `@intl-ai/unplugin`. Ele executa o binário `intl-ai`, que o pacote npm `intl-ai` instala automaticamente.

## Nota sobre a sintaxe do i18next

O i18next usa `{{variable}}` para interpolação (não ICU `{variable}`). Por exemplo:

```json
{
  "greeting": "Hello, {{name}}!",
  "items": "You have {{count}} items"
}
```

## Configuração

Crie um `intl-ai.toml` na raiz do seu projeto. Consulte [Configuration](/pt/guide/configuration) para o esquema completo.

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

Omita `processor` (o padrão passthrough) para que os marcadores `{{var}}` do i18next sobrevivam intactos.

## Utilização da aplicação React

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

### Utilizando traduções

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
