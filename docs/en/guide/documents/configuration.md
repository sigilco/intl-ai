---
title: Documents configuration
description: intl-ai-docs.toml reference. Experimental.
---

# Documents configuration

`intl-ai-docs` reads its own config file, `intl-ai-docs.toml` (or `.json`/`.yaml`), discovered by walking up from the working directory. Keep it next to the docs tree it manages; it is deliberately separate from `intl-ai.toml` so catalog and document pipelines never share settings.

Everything in `intl-ai.toml` that is not document-shaped (locale files, shims, corpus checks) stays there. The provider arms are identical to the corpus config: anything from [Providers](/guide/providers/) works verbatim in `intl-ai-docs.toml`.

## Top level

| Key                   | Type   | Default  | Description                                                                                                                     |
| --------------------- | ------ | -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `experimental`        | bool   | `false`  | Required. Every command refuses to run until this is `true`.                                                                    |
| `layout`              | string | `"flat"` | Tree arrangement: `locale-dirs` puts the source at `source_dir/<source>`; `flat` puts source files directly under `source_dir`. |
| `source_dir`          | path   | `"docs"` | Document root, resolved relative to the config file.                                                                            |
| `source`              | string | `"en"`   | Source locale id.                                                                                                               |
| `targets`             | list   | required | Target locales; each writes to `source_dir/<locale>`.                                                                           |
| `format`              | string | `"mdx"`  | DocFormat name. `"mdx"` (markdown included) is the only shipped format; `"markdown"` is an alias.                               |
| `batch_size`          | int    | engine   | Segments per translate request. Keep it small for long docs to stay under provider output limits.                               |
| `max_retries`         | int    | `3`      | Provider retries on retryable failures.                                                                                         |
| `glossary`            | map    | `{}`     | Term pins, same semantics as the corpus glossary.                                                                               |
| `locale_instructions` | map    | `{}`     | Per-locale style/dialect notes; resolved exact, language subtag, then `*`.                                                      |
| `exclude`             | list   | `[]`     | Extra globs (document-relative) to skip, on top of hidden dirs, `node_modules`, and target locale dirs.                         |
| `version`             | int    | —        | Config schema version; only `1` is understood.                                                                                  |

## `[provider]`

Same shape as `intl-ai.toml`: `kind = "http" | "command" | "replay"`, plus provider-specific fields. `api_key` accepts `${env:VAR}` and `${file:PATH}` interpolation.

## `[links]`

| Key       | Type   | Default       | Description                                                 |
| --------- | ------ | ------------- | ----------------------------------------------------------- |
| `rewrite` | bool   | `true`        | Insert the locale prefix into `/`-rooted link destinations. |
| `prefix`  | string | `"/{locale}"` | Prefix template; `{locale}` is replaced per target.         |

Links are rewritten at merge time only; shards store the unprefixed form, so changing `prefix` later rewrites destinations cleanly on the next fill.

## `[frontmatter]`

| Key    | Type | Default                    | Description                                             |
| ------ | ---- | -------------------------- | ------------------------------------------------------- |
| `keys` | list | `["title", "description"]` | Dotted frontmatter paths whose scalar values translate. |

## `[fill]`

| Key        | Type | Default | Description                                                                          |
| ---------- | ---- | ------- | ------------------------------------------------------------------------------------ |
| `validate` | list | `[]`    | Doc checks run as a shift-left gate per batch: `doc:protected` and `doc:corruption`. |

## `[check]` and `[[checks]]`

`[check]` carries the same options as corpus check: `fail_on` (finding kinds that exit non-zero, default `missing`, `stale`, `invalid`) and `cache` (incremental findings cache, default `true`). `[[checks]]` adds corpus-style spec/exec/judge entries on top of the always-on document builtins, which verify skeleton parity, protected regions, link prefixes, and rekey consistency.

## `[quality]`

Same LLM-as-a-judge block as `intl-ai.toml` (`enabled`, `provider`, thresholds). Scores land in the lockfile shards under `quality`.

## Lockfile

`intl-ai-docs.lock.d/` sits next to the config file. Each translated document gets a shard at `intl-ai-docs.lock.d/<locale>/<relpath>.toml` with per-segment `sourceHash`, `origin` (`ai` or `human`), `reviewed`, and `quality`. An incremental `check-cache.json` also lives there; gitignore it.
