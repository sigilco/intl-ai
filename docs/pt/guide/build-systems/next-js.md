---
title: Next.js
description: Tradução AI em tempo de compilação para i18n do Next.js. Sem sobrecarga em tempo de execução, qualquer modelo AI.
---

# Next.js

O `@intl-ai/next` envolve o seu `next.config` para que o binário `intl-ai` execute o `fill` antes do início da compilação. Funciona com ambos os bundlers do Next.js, webpack e Turbopack; consulte a [documentação do Next.js sobre Turbopack](https://nextjs.org/docs/app/api-reference/config/next-config-js/turbopack) para saber qual o seu utiliza. Versão mínima suportada: Next.js 14+.

## Instalação

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

## Configuração

Crie um `intl-ai.toml` na raiz do seu projeto. Consulte [Configuração](/pt/guide/configuration) para o esquema completo.

```typescript
import { withIntlAi } from "@intl-ai/next";

export default withIntlAi({
  reactStrictMode: true,
});
```

Não são necessárias alterações no código da sua aplicação. O `intl-ai fill` é executado enquanto o Next.js avalia a configuração, antes do bundler iniciar, para que as traduções sejam aplicadas em tempo de compilação sem sobrecarga em tempo de execução.

As opções de tradução vão no segundo argumento e espelham as opções do [`@intl-ai/unplugin`](/pt/guide/build-systems/):

```typescript
export default withIntlAi(
  { reactStrictMode: true },
  {
    failOn: ["missing", "invalid"], // build fails on these findings
    dev: false, // skip during `next dev`
  },
);
```

## Alternativa: adaptador unplugin

Se preferir o hook do bundler em vez do wrapper de configuração (ou não quiser a dependência), use o `@intl-ai/unplugin/webpack` diretamente no seu callback do webpack:

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

Combine com uma biblioteca i18n de [bibliotecas i18n](/pt/guide/i18n-libraries/).
