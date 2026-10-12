---
title: Configuración de documentos
description: Referencia de intl-ai-docs.toml. Experimental.
---

# Configuración de documentos

`intl-ai-docs` lee su propio archivo de configuración, `intl-ai-docs.toml` (o `.json`/`.yaml`), que se descubre recorriendo hacia arriba desde el directorio de trabajo. Colócalo junto al árbol de documentos que gestiona; está deliberadamente separado de `intl-ai.toml` para que los pipelines de catálogos y de documentos nunca compartan ajustes.

Todo lo que en `intl-ai.toml` no tiene forma de documento (archivos de locale, shims, checks de corpus) se queda ahí. Los arms de proveedor son idénticos a los de la configuración de corpus: cualquier cosa de [Proveedores](/es/guide/providers/) funciona tal cual en `intl-ai-docs.toml`.

## Nivel superior

| Key                    | Type       | Default       | Description                                                                                                                              |
| ---------------------- | ---------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `experimental`         | bool       | `false`       | Obligatorio. Todos los comandos se niegan a ejecutarse hasta que sea `true`.                                                                             |
| `layout`               | string     | `"flat"`      | Disposición del árbol: `locale-dirs` coloca la fuente en `source_dir/<source>`; `flat` coloca los archivos fuente directamente bajo `source_dir`.            |
| `source_dir`           | path       | `"docs"`      | Raíz de documentos, resuelta relativa al archivo de configuración.                                                                                     |
| `source`               | string     | `"en"`        | Id de la locale fuente.                                                                                                                         |
| `targets`              | list       | required      | Locales destino; cada una escribe en `source_dir/<locale>`.                                                                                    |
| `format`               | string     | `"mdx"`       | Nombre del DocFormat. `"mdx"` (markdown incluido) es el único formato publicado; `"markdown"` es un alias.                                          |
| `batch_size`           | int        | engine        | Segmentos por petición de traducción. Manténlo pequeño para documentos largos y no superar los límites de salida del proveedor.                                         |
| `max_retries`          | int        | `3`           | Reintentos de proveedor en fallos reintentables.                                                                                                   |
| `glossary`             | map        | `{}`          | Términos fijados, con la misma semántica que el glosario de corpus.                                                                                          |
| `locale_instructions`  | map        | `{}`          | Notas de estilo/dialecto por locale; se resuelven por locale exacta, subetiqueta de idioma y luego `*`.                                                                |
| `exclude`              | list       | `[]`          | Globs extra (relativos al documento) a omitir, además de directorios ocultos, `node_modules` y los directorios de locales destino.                                   |
| `version`              | int        | —             | Versión del esquema de configuración; solo se entiende `1`.                                                                                            |

## `[provider]`

Misma forma que `intl-ai.toml`: `kind = "http" | "command" | "replay"`, más los campos específicos del proveedor. `api_key` acepta la interpolación `${env:VAR}` y `${file:PATH}`.

## `[links]`

| Key       | Type   | Default        | Description                                                            |
| --------- | ------ | -------------- | ---------------------------------------------------------------------- |
| `rewrite` | bool   | `true`         | Inserta el prefijo de locale en los destinos de enlace con raíz `/`.            |
| `prefix`  | string | `"/{locale}"`  | Plantilla de prefijo; `{locale}` se sustituye por cada destino.                    |

Los enlaces se reescriben solo en el merge; los shards guardan la forma sin prefijo, así que cambiar `prefix` más tarde reescribe los destinos limpiamente en el próximo fill.

## `[frontmatter]`

| Key    | Type | Default                     | Description                                          |
| ------ | ---- | --------------------------- | ---------------------------------------------------- |
| `keys` | list | `["title", "description"]`  | Rutas de frontmatter (con puntos) cuyos valores escalares se traducen. |

## `[fill]`

| Key        | Type | Default | Description                                                                                    |
| ---------- | ---- | ------- | ---------------------------------------------------------------------------------------------- |
| `validate` | list | `[]`    | Checks de documento ejecutados como puerta shift-left por lote: `doc:protected` y `doc:corruption`.           |

## `[check]` y `[[checks]]`

`[check]` lleva las mismas opciones que el check de corpus: `fail_on` (tipos de hallazgo que salen con código distinto de cero, por defecto `missing`, `stale`, `invalid`) y `cache` (caché incremental de hallazgos, por defecto `true`). `[[checks]]` añade entradas estilo corpus spec/exec/judge además de los builtins de documento siempre activos, que verifican la paridad del esqueleto, las regiones protegidas, los prefijos de enlaces y la consistencia del rekey.

## `[quality]`

Mismo bloque de LLM-as-a-judge que `intl-ai.toml` (`enabled`, `provider`, umbrales). Las puntuaciones quedan en los shards del lockfile bajo `quality`.

## Lockfile

`intl-ai-docs.lock.d/` se sitúa junto al archivo de configuración. Cada documento traducido tiene un shard en `intl-ai-docs.lock.d/<locale>/<relpath>.toml` con `sourceHash`, `origin` (`ai` o `human`), `reviewed` y `quality` por segmento. También vive ahí un `check-cache.json` incremental; añádelo a gitignore.
