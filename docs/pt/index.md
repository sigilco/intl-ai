---
title: intl-ai
description: Tradução i18n com tecnologia de IA no tempo de compilação. Um binário, qualquer bundler, qualquer modelo. Zero dependências de tempo de execução.
---

# intl-ai

Tradução i18n com IA no tempo de compilação. Zero tempo de execução, qualquer bundler, qualquer modelo.

[Começar](/pt/guide/getting-started/) · [GitHub](https://github.com/sigilco/intl-ai) · [Patrocinador](https://buy.polar.sh/polar_cl_Mv1gdlG7bw3I70EC9IHtfeSHJj4PEKvA7JAUz23CFhj)

::: info
O <a href="/demo/" data-spa-ignore>demonstração ao vivo</a> executa o pipeline de preenchimento em WebAssembly: solte um arquivo de localidade, observe as chaves serem transmitidas, nenhuma instalação necessária.
:::

## Por que intl-ai

- **Any bundler**: Vite, Webpack, Rollup, esbuild, Rspack, Rolldown, Farm e Next.js. Um plugin, todas as ferramentas de build.
- **Any model**: OpenAI, Anthropic, Ollama ou qualquer API compatível com OpenAI. Configure uma vez, traduza em todo lugar.
- **Zero runtime**: Todas as traduções ocorrem no tempo de compilação. Nenhum SDK de IA em produção, nenhuma latência, nenhum bloqueio de fornecedor.
- **Cross-platform**: Integrações nativas Expo, Flutter, SwiftUI e .NET. Envie traduções para qualquer build móvel ou de desktop.

## Instalação rápida

::: tabs

== tab "Homebrew"

```sh
brew install sigilco/tap-intl-ai/intl-ai
```

== tab "mise"

```sh
mise use npm:intl-ai@latest
```

== tab "install script"

```sh
curl -fsSL https://intl-ai.illo.fyi/install.sh | sh
```

:::
