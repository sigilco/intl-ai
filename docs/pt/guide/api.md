---
title: Referência da CLI
description: "Referência de comandos do intl-ai: fill, check, mark, review, lockfile, config, status."
---

# Referência da CLI

`intl-ai` é um binário único. Todos os comandos aceitam `--format json` para saída legível por máquina, e `-` como `--config` lê TOML a partir da entrada padrão.

## `intl-ai init`

Gerar estrutura do `intl-ai.toml` para o projeto atual.

```bash
intl-ai init --locale-dir locales --source en --targets es,fr
```

## `intl-ai fill`

Traduzir chaves em falta. Aditivo por predefinição: nunca sobrescreve valores existentes a menos que `--regenerate` ou `--stale` o indiquem.

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

`--regenerate` é destrutivo: requer um escopo `--keys`, `--keys-file` ou `--yes` para confirmar uma reescrita completa da localidade.

## `intl-ai check`

Reportar descobertas sem escrever. O estado de saída segue `--fail-on`.

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

As descobertas são armazenadas em cache incrementalmente em `.intl-ai/check-cache.json`: as chaves inalteradas reproduzem os resultados anteriores (`cached: true` na saída JSON) em vez de volver a executar a verificação.

## `intl-ai mark`

Definir a origem nas entradas do lockfile (a escapatória para a regra de revisão posicional).

```bash
intl-ai mark 'checkout.*' --locale es --origin ai      # rebaseline as AI-owned
intl-ai mark 'checkout.*' --locale es --origin human   # claim as human-owned
intl-ai mark 'legal.*' --locale es --absent            # tombstone: never translate
intl-ai mark 'legal.*' --locale es --present           # clear a tombstone
```

`--origin ai` requer uma entrada existente; `--origin human` cria ou atualiza uma entrada. As chaves tombstoned são ignoradas por `fill` e não são reportadas como ausentes por `check`.

## `intl-ai review` / `intl-ai unreview`

Definir `reviewed` nas entradas do lockfile.

```bash
intl-ai review 'checkout.*' --locale es
intl-ai unreview 'auth.*' --locale es
```

## `intl-ai status`

Contagens de inventário por localidade. Somente leitura e suficientemente barato para que os agentes o chamem antes de decidir o que fazer.

```bash
intl-ai status
intl-ai status --locale es
```

## `intl-ai lockfile`

Manutenção do lockfile sobre os fragmentos `intl-ai.lock.d/`.

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

O schema comprometido encontra-se em `docs/public/schema/intl-ai.schema.json` e é servido em `https://intl-ai.illo.fyi/schema/v1.json`.

## Flags globais

| Sinalizador       | Descrição                                                                   |
| ----------------- | --------------------------------------------------------------------------- |
| `--config <path>` | Caminho do arquivo de configuração. `-` lê TOML a partir da entrada padrão. |
| `--format json`   | Saída legível por máquina (também `human`, o padrão).                       |
