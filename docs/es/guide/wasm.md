---
title: Demostración del navegador y compilación wasm
description: Run the intl-ai pipeline in the browser via the wasm build, or embed the crate in your tooling.
---

# Demostración del navegador y compilación wasm

El crate `intl-ai-wasm` compila las piezas puras del pipeline (flatten,
construcción de prompts, parseo de respuestas, chequeos de reglas
internos, unflatten) a `wasm32-unknown-unknown` con `wasm-bindgen`.
El JavaScript host opera cada operación IO: lectura de archivos, el bucle
fetch contra un proveedor compatible con OpenAI, reintentos, y batcheo.

## Prueba la demostración

Una página de demostración estática vive en `demo/wasm/`.
Suelte o pegue un archivo de configuración regional de origen, establezca las configuraciones regionales de origen y destino, y
la página rellena las claves faltantes, ejecuta las comprobaciones y muestra el archivo relleno con un enlace de descarga.
Cada llamada wasm se ejecuta dentro de `worker.mjs`, un Web Worker de módulo, así que
el trabajo de parseo/comprobación nunca bloquea la página; el bucle de obtención, los reintentos y el procesamiento por lotes permanecen en el hilo principal.

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

Luego sirve el directorio sobre http (los módulos ES no se cargan desde `file://`):

```sh
cd demo/wasm && python3 -m http.server 8080
```

## SPA de estudio

`demo/web/` es un SPA Svelte más completo en el mismo crate: una tarjeta
de proveedor colapsible (presets, obtención de `/models`, botón de
prueba, persistencia `localStorage`), selects de locale fuente y
objetivo con autodetect de nombre de archivo, relleno en streaming que
llega fila por fila, un indicador de progreso lineal, y una vista de
juez por clave con razones expandibles y hallazgos de chequeo.
Ve `demo/web/README.md` para los comandos (`pnpm dev`,
`scripts/sync-wasm.sh`).

## Provider

The demo posts the request bodies the wasm build produces to
`{baseUrl}/chat/completions`, byte-identical to what the CLI's `http`
transport sends.
The default is `https://api.illo.fyi/v1` with model `illo-demo`, the
same gateway the [just-ai-illo](https://github.com/espetro/just-ai-illo)
project uses.

::: warning The demo must run on an _.illo.fyi origin for the default provider
`api.illo.fyi` solo responde solicitudes CORS de subdominios `_.illo.fyi`, y
su`access-control-allow-headers`no incluye`authorization`.
Una página servida localmente debe apuntar el campo URL base a un
endpoint OpenAI-compatible diferente (`demo/wasm/mock-provider.mjs`
 incluye un mock local pequeño para desarrollo).
Una clave API solo se envía cuando el campo está lleno.
:::

## Qué expone la API de wasm

| Función                              | Purpose                                           |
| ------------------------------------- | ------------------------------------------------- |
| `flatten(sourceJson)`                 | Locale file text (JSON or YAML) to a flat map     |
| `missingKeys(sourceFlat, targetFlat)` | Claves fuente ausentes del objetivo, ordenadas        |
| `buildTranslateBody(req)`             | Cuerpo de completaciones de chat para un lote de traducción     |
| `parseTranslations(content)`          | Contenido de respuesta a un mapa `key -> translated`     |
| `buildJudgeBody(req)`                 | Cuerpo del lote de calidad adversarial                    |
| `parseJudgements(content)`            | Puntuaciones, razones y listas de errores por clave          |
| `runChecks(items, checkIds, ctx)`     | `icu`, `placeholder-parity`, `dialect:*` findings |
| `unflatten(targetJson, map)`          | Merge flat keys into the target and serialize it  |
| `runFill(spec, translate, onEvent)`   | Rellenar claves faltantes, `ProgressEvent`s en streaming     |
| `runCheck(spec, judge, onEvent)`      | Comprobar un archivo de destino, transmitiendo `ProgressEvent`s   |

`judge` is deliberately not inside `runChecks`: it needs a provider round
trip, which the host performs with `buildJudgeBody`/`parseJudgements`.

## Controladores de streaming

`runFill` y `runCheck` ejecutan el corte demo de los pipelines dentro de wasm
y reenvían el flujo `ProgressEvent` principal de `crates/intl-ai-core`
(`run_started`, `batch_started`, `batch_finished`, `key_done`, `finding`,
`locale_finished`, `run_finished`) a un callback de host como objetos JS planos:

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

The transport callbacks keep IO host-side: the host posts the body to
`{baseUrl}/chat/completions` and resolves `choices[0].message.content`.
A rejected promise terminal-fails that batch's keys; later batches still
run, matching the CLI's per-batch failure policy.

## Differences from the CLI

The wasm build is a demo cut, not a port of `fill`:

- No lockfile shards, staleness detection, or provenance: the demo
  translates "missing keys" only.
- No config discovery, stat/check caches, or shard locks: those are
  filesystem-bound and stay native-only.
- No hay chequeos `spec` o `exec`, y Ningún transporte `command`/`replay`/`http`
  : `std::process` y la pila TLS no existen en
  `wasm32-unknown-unknown`, así que los proveedores los exponen detrás de
  características cargo que el crate wasm omite.
- Retry/backoff lives in the host JavaScript, which also owns batching
  and progress.

## Lintar la compilación en CI

`cargo check -p intl-ai-wasm --target wasm32-unknown-unknown` runs in CI
(`.github/workflows/ci.yml`).
It catches accidental filesystem or transport use in the wasm graph at
compile time rather than as a runtime trap in the browser.
