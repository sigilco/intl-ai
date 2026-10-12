---
title: Demonstração no navegador e compilação wasm
description: Execute o pipeline intl-ai no navegador através da compilação wasm, ou incorpore a crate nas suas ferramentas.
---

# Demonstração no navegador e compilação wasm

O crate `intl-ai-wasm` compila as peças puras do pipeline (flatten,
construção de prompt, análise de resposta, as verificações de regras incorporadas, unflatten)
para `wasm32-unknown-unknown` com `wasm-bindgen`.
O JavaScript do host possui todas as operações de E/S: leitura de arquivos, o loop
fetch contra um provedor compatível com OpenAI, retentativas e loteamento.

## Try the demo

Uma página de demonstração estática está em `demo/wasm/`.
Descarte ou cole um arquivo de localidade de origem, defina as localidades de origem e destino, e a página preenche as chaves faltantes, executa as verificações e renderiza o arquivo preenchido com um link de download.
Cada chamada WASM roda dentro do `worker.mjs`, um módulo Web Worker, então o trabalho de parse/verificação nunca bloqueia a página; o loop de fetch, reintentos e batching permanecem na thread principal.

::: tabs

== tab "wasm-bindgen-cli"

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
./demo/wasm/build.sh
```

== tab "wasm-pack"

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build crates/intl-ai-wasm --target web --out-dir ../../demo/wasm/pkg
```

:::

Then serve the directory over http (ES modules do not load from `file://`):

```sh
cd demo/wasm && python3 -m http.server 8080
```

## Studio SPA

`demo/web/` é uma SPA Svelte mais completa no mesmo crate: um cartão de fornecedor recolhível (predefinições, `/models` pull, botão de teste, `localStorage`
persistence), seleções de local de origem e destino com deteção automática de nome de ficheiro, preenchimento transmitido que chega linha por linha, um indicador de progresso linear e uma visualização de juiz por chave com motivos e descobertas de verificação expansíveis.
Consulte `demo/web/README.md` para os comandos (`pnpm dev`,
`scripts/sync-wasm.sh`)

## Fornecedor

O demo publica os corpos de solicitação que a compilação wasm produz em
`{baseUrl}/chat/completions`, idênticos em bytes ao que o transporte `http` do CLI envia.
O padrão é `https://api.illo.fyi/v1` com o modelo `illo-demo`, o
mesmo gateway que o projeto [just-ai-illo](https://github.com/espetro/just-ai-illo)
utiliza.

::: warning The demo must run on an _.illo.fyi origin for the default provider
`api.illo.fyi` apenas responde a pedidos CORS de subdomínios `_.illo.fyi`, e¬no seu `access-control-allow-headers`não inclui`authorization`.¬Uma página servida localmente deve apontar o campo URL base para um¬endpoint compatível com OpenAI diferente (`demo/wasm/mock-provider.mjs` inclui um pequeno¬mock local para desenvolvimento).¬Uma chave de API só é enviada quando o campo está preenchido.
:::

## O que a API WASM expõe

| Função                                | Objetivo                                                      |
| ------------------------------------- | ------------------------------------------------------------- |
| `flatten(sourceJson)`                 | Texto do ficheiro de locale (JSON ou YAML) para um mapa plano |
| `missingKeys(sourceFlat, targetFlat)` | Chaves de origem ausentes no destino, ordenadas               |
| `buildTranslateBody(req)`             | Chat-completions body for a translation batch                 |
| `parseTranslations(content)`          | Response content to a `key -> translated` map                 |
| `buildJudgeBody(req)`                 | Corpo de lote de qualidade adversarial                        |
| `parseJudgements(content)`            | Pontuações por chave, motivos e listas de erros               |
| `runChecks(items, checkIds, ctx)`     | Descobertas sobre `icu`, `placeholder-parity`, `dialect:*`    |
| `unflatten(targetJson, map)`          | Mesclar chaves planas no destino e serializá-lo               |
| `runFill(spec, translate, onEvent)`   | Fill missing keys, streaming `ProgressEvent`s                 |
| `runCheck(spec, judge, onEvent)`      | Verifique um arquivo alvo, streamando `ProgressEvent`s        |

`judge` não está intencionalmente dentro de `runChecks`: precisa de um ciclo de ida e volta do provedor,
que o host realiza com `buildJudgeBody`/`parseJudgements`.

## Streaming drivers

`runFill` e `runCheck` executam a demonstração de corte dos pipelines dentro do wasm
e encaminham o fluxo central `ProgressEvent` de `crates/intl-ai-core`
(`run_started`, `batch_started`, `batch_finished`, `key_done`, `finding`,
`locale_finished`, `run_finished`) para um callback de host como objetos JS simples:

```ts
runFill(
  {
    source, // source locale file text (JSON or YAML)
    target, // optional existing target file text
    sourceLocale,
    targetLocale,
    model,
    batchSize, // optional, default 20
  },
  translate, // (chatCompletionsBody) => Promise<contentString>
  onEvent, // (event: {type: string, ...}) => void
); // -> Promise<{filled, output, failures}>

runCheck(
  {
    source,
    target, // target = file text to check (e.g. fill output)
    sourceLocale,
    targetLocale,
    model,
    checkIds, // ["icu", "placeholder-parity", "dialect:<locale>"]
    judgeItems, // optional [{key, locale, source, translation}]
  },
  judge, // same transport shape as translate, or null
  onEvent,
); // -> Promise<{findings, judgements, errors}>
```

Os callbacks de transporte mantêm a E/S do lado do host: o host envia o corpo para¬`{baseUrl}/chat/completions` e resolve `choices[0].message.content`.¬Uma promise rejeitada faz falhar terminalmente as chaves desse lote; lotes posteriores ainda¬executam, correspondendo à política de falha por lote da CLI.

## Diferenças em relação ao CLI

A compilação wasm é uma versão de demonstração, não um port de `fill`:

- Sem fragmentos de lockfile, detecção de desatualização ou proveniência: o demo
  traduz apenas "chaves ausentes".
- Sem descoberta de configuração, caches de stat/check, ou bloqueios de shard: esses são`n dependentes do sistema de ficheiros e permanecem apenas nativos.
- Não há verificações `spec` ou `exec`, e nenhum transporte `command`/`replay`/`http`
  : `std::process` e a pilha TLS não existem em
  `wasm32-unknown-unknown`, portanto os provedores os expõem atrás de recursos
  cargo que o crate wasm omite.
- A lógica de retry/backoff está no JavaScript host, que também gerencia o agrupamento
  e o progresso.

## Lint the build in CI

`cargo check -p intl-ai-wasm --target wasm32-unknown-unknown` é executado no CI
(`.github/workflows/ci.yml`).
Deteta o uso acidental de sistema de ficheiros ou transporte no grafo wasm em
tempo de compilação, em vez de como uma armadilha em tempo de execução no navegador.
