---
title: Vue (vue-i18n)
description: Traduza localmente o vue-i18n no tempo de construção. Funciona com qualquer modelo de IA, custo zero em tempo de execução.
---

# Vue (vue-i18n)

Este guia cobre o consumo do vue-i18n. Para configuração do bundler, veja [Sistemas de Build](/pt/guide/build-systems/).

## Visão Geral

O intl-ai gera arquivos JSON de tradução no tempo de construção. O vue-i18n consome esses arquivos em tempo de execução na sua aplicação Vue. Este guia mostra como configurar ambas as ferramentas juntas.

## Instalação

Instale o intl-ai e o vue-i18n:

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

Você só precisa de `@intl-ai/unplugin`. Ele executa o binário `intl-ai`, que o pacote npm `intl-ai` instala automaticamente.

## Configuração

Crie um `intl-ai.toml` na raiz do seu projeto. Veja [Configuração](/pt/guide/configuration) para o esquema completo.

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

## Configuração da Aplicação Vue

### Estrutura dos Arquivos de Localização

O intl-ai gera arquivos JSON como:

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

### Criar Instância i18n

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

### Usando Traduções

Nas plantas Vue:

```vue
<template>
  <p>{{ $t("greeting", { name: "World" }) }}</p>
  <p>{{ $t("items.one", { count: 5 }) }}</p>
</template>
```

Na API de Composição:

```vue
<script setup>
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>
```

## Nota do Processador

O vue-i18n suporta ICU MessageFormat. Defina `processor = "icu"` em `intl-ai.toml` para que as traduções geradas por IA preservem corretamente os placeholders do ICU.
