---
title: CLI reference
description: "intl-ai command reference: fill, check, mark, review, lockfile, config, status, migrate."
---

# CLI reference

`intl-ai` is a single binary. Every command accepts `--format json` for machine-readable output, and `-` as `--config` reads TOML from stdin.

## `intl-ai init`

Scaffold `intl-ai.toml` for the current project.

```bash
intl-ai init --locale-dir locales --source en --targets es,fr
```

## `intl-ai fill`

Translate missing keys. Additive by default: it never overwrites existing values unless `--regenerate` or `--stale` say so.

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
```

`--regenerate` is destructive: it requires a `--keys` scope, `--keys-file`, or `--yes` to confirm a whole-locale rewrite.

## `intl-ai check`

Report findings without writing. Exit status follows `--fail-on`.

```bash
intl-ai check                          # all configured checks and targets
intl-ai check --fail-on missing,invalid
intl-ai check --fail-on none           # report only, never fail
intl-ai check --locale es --keys 'auth.*'
intl-ai check --origin ai              # scope stale/modified/unreviewed to one origin
intl-ai check --self-test              # run each spec check's fixtures
intl-ai check --no-cache               # skip stat cache and the findings cache
```

Findings are cached incrementally in `.intl-ai/check-cache.json`: unchanged keys replay their previous findings (`cached: true` in JSON output) instead of re-running the check.

## `intl-ai mark`

Set the origin on lockfile entries (the escape hatch for the positional review rule).

```bash
intl-ai mark 'checkout.*' --locale es --origin ai      # rebaseline as AI-owned
intl-ai mark 'checkout.*' --locale es --origin human   # claim as human-owned
intl-ai mark 'legal.*' --locale es --absent            # tombstone: never translate
intl-ai mark 'legal.*' --locale es --present           # clear a tombstone
```

`--origin ai` requires an existing entry; `--origin human` upserts one. Tombstoned keys are skipped by `fill` and not reported missing by `check`.

## `intl-ai review` / `intl-ai unreview`

Set `reviewed` on lockfile entries.

```bash
intl-ai review 'checkout.*' --locale es
intl-ai unreview 'auth.*' --locale es
```

## `intl-ai status`

Per-locale inventory counts. Read-only and cheap enough for agents to call before deciding what to do.

```bash
intl-ai status
intl-ai status --locale es
```

## `intl-ai lockfile`

Lockfile maintenance over the `intl-ai.lock.d/` shards.

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

The committed schema lives at `docs/public/schema/intl-ai.schema.json` and is served at `https://intl-ai.pages.dev/schema/v1.json`.

## `intl-ai migrate`

Import a 0.4.x `intl-ai.lock.json` into `intl-ai.lock.d/` shards.

## Global flags

| Flag              | Description                                          |
| ----------------- | ---------------------------------------------------- |
| `--config <path>` | Config file path. `-` reads TOML from stdin.         |
| `--format json`   | Machine-readable output (also `human`, the default). |
