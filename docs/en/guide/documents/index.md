---
title: Localize documents
description: Translate markdown and MDX document trees with intl-ai-docs. Experimental.
---

# Localize documents

`intl-ai-docs` is a second binary that ships with every `intl-ai` install. It translates whole document trees (markdown and MDX) the same way `intl-ai` translates locale files: a source tree in one language, a generated tree per target locale, and a lockfile that records what came from where.

::: warning Experimental
Document support is opt-in. Every `intl-ai-docs` command refuses to run unless your config sets `experimental = true`. The MDX reader handles the constructs real docs sites use (frontmatter, code fences, callouts, JSX blocks), but the format is new: run `intl-ai-docs check` in CI before publishing.
:::

## Install

You already have it. The install script, Homebrew formula, npm package, and mise all install both `intl-ai` and `intl-ai-docs`. Verify with `intl-ai-docs --help`.

## How it works

Instead of keys and values, `intl-ai-docs` splits each document into a **skeleton** and **segments**:

1. **Extract**: the MDX parser walks the document and emits a skeleton (the document structure with placeholder IDs) plus one segment per translatable span: headings, paragraphs, list items, table cells, and frontmatter strings.
2. **Fill**: segments are translated in batches by your provider, exactly like `intl-ai fill` handles catalog keys.
3. **Merge**: translated segments are spliced back into the skeleton by byte range. Structure, comments, code fences, inline code, and MDX components are preserved byte-for-byte because they were never inside a segment.

Protected regions (code blocks, inline code, commands, URLs) cannot be corrupted by construction: the model never sees them as translatable text, and the write gate verifies parity before anything is written.

## Quickstart

Create `intl-ai-docs.toml` next to your docs tree:

```toml
experimental = true
layout = "locale-dirs"      # source at docs/en, targets at docs/<locale>
source_dir = "docs"
source = "en"
targets = ["es", "pt"]
batch_size = 8

[provider]
kind = "http"
provider = "openai"
base_url = "${env:AI_BASE_URL}"
model = "${env:AI_MODEL_ID}"
api_key = "${env:AI_API_KEY}"
```

Then fill and check:

```bash
intl-ai-docs fill     # writes docs/es/** and docs/pt/**
intl-ai-docs check    # read-only report; --fail-on invalid for CI
```

Translated documents land in `docs/<locale>/` mirroring the source tree. Provenance lives in `intl-ai-docs.lock.d/<locale>/<doc>.toml` shards with the same fields as the corpus lockfile (`origin`, `sourceHash`, `reviewed`, `quality`). Commit both the trees and the shards.

## Layouts

`layout` controls where the source tree lives relative to its targets:

- `locale-dirs`: every locale is a sibling directory, including the source (`docs/en`, `docs/es`, `docs/pt`). This matches docmd i18n and most docs frameworks that want a directory per locale.
- `flat`: source files sit directly under `source_dir`, and target trees are written to `source_dir/<locale>`. Use this when your source files cannot live in a locale directory.

Links are rewritten per locale by default: `/`-rooted destinations get the target's prefix (`/es/guide/...`) so translated pages link inside their own tree. Tune or disable with `[links]`.

## What stays in English

- Code fences, inline code, commands, flags, file paths, and URLs.
- MDX/JSX components and fenced callout syntax; only their prose children translate.
- Frontmatter keys other than those listed in `frontmatter.keys` (default `title`, `description`).
- Sidebar labels: nav lives in site config, not documents. For docmd, drop a `navigation.json` in each locale directory.

Add a `[locale_instructions]` entry per locale to steer terminology, dialect, and register; the resolution rule is the same as `intl-ai` (exact locale, language subtag, then `*`).

::: tip Keep backticks verbatim
Models sometimes translate inside inline code even though they should not. Spell it out in `locale_instructions`, for example: `Never translate text inside backticks; keep code blocks, commands, and identifiers verbatim.`
:::

See [Configuration](/guide/documents/configuration/) for the full `intl-ai-docs.toml` reference.
