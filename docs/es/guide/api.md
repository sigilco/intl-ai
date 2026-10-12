---
title: Referencia de la CLI
description: "Referencia de comandos intl-ai: fill, check, mark, review, lockfile, config, status."
---

# Referencia de la CLI

`intl-ai` es un binario único. Todos los comandos aceptan `--format json` para salida legible por máquina, y `-` como `--config` lee TOML desde la entrada estándar.

## `intl-ai init`

Crear la estructura base de `intl-ai.toml` para el proyecto actual.

```bash
intl-ai init --locale-dir locales --source en --targets es,fr
```

## `intl-ai fill`

Traducir las claves que faltan. Aditivo por defecto: nunca sobrescribe valores existentes a menos que `--regenerate` o `--stale` lo indiquen.

```bash
intl-ai fill                          # all missing keys, all configured targets
intl-ai fill --locale es              # one locale (repeatable or comma-separated)
intl-ai fill --keys 'auth.*'          # glob scope (repeatable or comma-separated)
intl-ai fill --keys-file keys.txt     # one key glob per line
intl-ai fill --stale                  # re-fill AI-owned keys whose source changed
intl-ai fill --regenerate --keys 'x'  # rewrite AI-owned existing values
intl-ai fill --regenerate --yes       # confirm a whole-locale rewrite
intl-ai fill --regenerate --include-human --keys 'x'  # also human-owned values
intl-ai fill --dry-run                # report without writing anything
intl-ai fill --validate icu,judge     # run the fill gate with these checks
intl-ai fill --no-validate            # skip the configured [fill].validate gate
intl-ai fill --judge-threshold 0.9    # override the judge threshold for this run
intl-ai fill --no-cache               # skip the stat cache for this run
intl-ai fill --progress               # stream per-batch progress lines to stderr
```

`--regenerate` es destructivo: requiere un alcance `--keys`, `--keys-file` o `--yes` para confirmar una reescritura completa de la configuración regional.

## `intl-ai check`

Informar hallazgos sin escribir. El estado de salida sigue `--fail-on`.

```bash
intl-ai check                          # all configured checks and targets
intl-ai check --fail-on missing,invalid
intl-ai check --fail-on none           # report only, never fail
intl-ai check --locale es --keys 'auth.*'
intl-ai check --origin ai              # scope stale/modified/unreviewed to one origin
intl-ai check --self-test              # run each spec check's fixtures
intl-ai check --no-cache               # skip stat cache and the findings cache
intl-ai check --progress               # stream per-locale findings to stderr
```

Los hallazgos se almacenan en caché de forma incremental en `.intl-ai/check-cache.json`: las claves sin cambios reproducen sus hallazgos anteriores (`cached: true` en la salida JSON) en lugar de volver a ejecutar la comprobación.

## `intl-ai mark`

Establecer el origen en las entradas del lockfile (la vía de escape para la regla de revisión posicional).

```bash
intl-ai mark 'checkout.*' --locale es --origin ai      # rebaseline as AI-owned
intl-ai mark 'checkout.*' --locale es --origin human   # claim as human-owned
intl-ai mark 'legal.*' --locale es --absent            # tombstone: never translate
intl-ai mark 'legal.*' --locale es --present           # clear a tombstone
```

`--origin ai` requiere una entrada existente; `--origin human` la crea o actualiza. Las claves tombstone son omitidas por `fill` y no se reportan como faltantes por `check`.

## `intl-ai review` / `intl-ai unreview`

Establecer `reviewed` en las entradas del lockfile.

```bash
intl-ai review 'checkout.*' --locale es
intl-ai unreview 'auth.*' --locale es
```

## `intl-ai status`

Recuentos de inventario por configuración regional. Solo lectura y lo suficientemente económico para que los agentes lo llamen antes de decidir qué hacer.

```bash
intl-ai status
intl-ai status --locale es
```

## `intl-ai lockfile`

Mantenimiento del archivo de bloqueo sobre los fragmentos `intl-ai.lock.d/`.

```bash
intl-ai lockfile check   # validate every shard
intl-ai lockfile fmt     # rewrite shards to canonical form (idempotent)
intl-ai lockfile merge   # resolve merge conflicts in shards
```

## `intl-ai config`

```bash
intl-ai config validate   # resolve and validate the config (secrets masked)
intl-ai config schema     # print the JSON Schema for the config contract
```

El esquema confirmado reside en `docs/public/schema/intl-ai.schema.json` y se sirve en `https://intl-ai.illo.fyi/schema/v1.json`.

## Indicadores globales

| Bandera           | Descripción                                                                |
| ----------------- | -------------------------------------------------------------------------- |
| `--config <path>` | Ruta del archivo de configuración. `-` lee TOML desde la entrada estándar. |
| `--format json`   | Salida legible por máquina (también `human`, el valor predeterminado).     |
