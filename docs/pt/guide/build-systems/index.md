---
title: Sistemas de compilação
description: O intl-ai suporta Vite, Webpack, Rollup, esbuild, Rspack, Rolldown e Farm através do unplugin. Escolha o seu empacotador.
---

# Sistemas de compilação

O intl-ai é executado em tempo de compilação através do `@intl-ai/unplugin`, um thin shim que inicia o binário `intl-ai` (instalado automaticamente como dependência; consulte [Introdução](/pt/guide/getting-started/) para outros canais de instalação e configuração do `intl-ai.toml`). Ele suporta todos os principais empacotadores através do [unjs/unplugin](https://github.com/unjs/unplugin). Escolha o seu empacotador abaixo.

As opções do plugin espelham os controlos de qualidade da CLI: `validate` (portão de preenchimento), `judgeThreshold`, `failOn` (severidades do `check --fail-on`), `dev` e `strict`. Consulte o [README do pacote](https://www.npmjs.com/package/@intl-ai/unplugin) para a tabela completa.

- [Vite](/pt/guide/build-systems/vite) - Ferramenta de construção moderna e rápida
- [Webpack](/pt/guide/build-systems/webpack) - Empacotador padrão da indústria
- [Rollup](/pt/guide/build-systems/rollup) - Empacotador de módulos flexível
- [esbuild](/pt/guide/build-systems/esbuild) - Empacotador de JavaScript extremamente rápido
- [Rspack](/pt/guide/build-systems/rspack) - Empacotador compatível com webpack baseado em Rust
- [Rolldown](/pt/guide/build-systems/rolldown) - Empacotador compatível com Rollup baseado em Rust
- [Farm](/pt/guide/build-systems/farm) - Ferramenta de construção web baseada em Rust
- [Next.js](/pt/guide/build-systems/next-js) - Framework React com ponte Turbopack

Se utilizar um framework, conecte o empacotador aqui e combine-o com uma biblioteca i18n de [bibliotecas i18n](/pt/guide/i18n-libraries/).
