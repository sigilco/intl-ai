# Plan: ship DocFormat tier — MDX (experimental)

## TL;DR

> **GitHub Issue**: #18
> **Project Item**: https://github.com/orgs/sigilco/projects/2 (item pending: project API write blocked for session token)
> **Depends**: #107 (release/shim), #108 (adopt), #109 (AsciiDoc, backlog)
> **Research**: `.agents/plans/2026-10-09-docs-format-research.md`
> **ADR**: `.agents/docs/adr/0001-markdown-companion-tool.md`

`DocFormat` is a sibling adapter to `Format`: `extract` produces a skeleton plus a `FlatMap` of segments, the existing engine translates it, `merge` splices bytes back.
Tier-1 impl is MDX; CommonMark/GFM/docmd markdown falls out by extension.
Ships as `intl-ai-docs` binary + `@intl-ai/docs` shim, gated behind an experimental flag; `intl-ai` binary is untouched.

## Workstreams

### W0: engine refactor (intl-ai-core)

- Lift the middle of `fill_locale` into `fill_flat(source: &FlatMap, target: &FlatMap, shard: &mut Shard, ...)` returning `LocaleFillResult` + failures; `fill` keeps corpus IO, calls it once per locale.
- Same for `check`: the per-locale batch/diff path already eats `CheckItem`s; expose a docs-callable `check_flat`.
- Lockfile: generalize shard addressing so docs can use `intl-ai-docs.lock.d/<locale>/<relpath>.toml`; add an optional `context`/`note` field on `Entry` if segment context needs storing (defer if not).
- Guard: `cargo test --workspace` green before moving on; zero behavior change in `intl-ai fill|check`.

### W1: `intl-ai-docs` crate (DocFormat + markdown/MDX impl)

- Types: `trait DocFormat { name, extensions, extract(&self, text, path) -> Result<Document>, merge(&self, doc, target: &FlatMap) -> Result<Vec<u8>> }`; `Document { skeleton, segments: BTreeMap<SegId, Segment> }`; `Segment { kind, text(masked), context }`.
- `SegId`: `h:<blake3-hex>` over normalized source text; position stored in skeleton for diagnostics.
- Markdown impl (`format = "mdx"`, flavors `mdx|markdown|docmd` as config flags where they differ):
  - `pulldown-cmark::into_offset_iter()` for block boundaries + byte spans (GFM ext: tables, tasklists, strikethrough, footnotes).
  - Line scanner runs first for opaque structures pulldown-cmark does not know: fenced code, math blocks, `:::` containers (known set extracts prose; unknown stays opaque + warn), JSX/HTML blocks, `import`/`export` lines.
  - Frontmatter: `---` YAML block parsed; allowlisted string leaves (`title`, `description`, `nav*`/`sidebar*`, configurable) become `fm.*` segments; splice by line-level value replacement.
  - Inline masking per segment: code spans, link destinations, image paths, autolinks, inline JSX become `⟦0⟧`-style tokens; mask map lives in the skeleton; unknown construct = mask, not corruption.
- `merge`: translate the FlatMap, restore masks inside each segment, verify every token survived exactly once, splice translated spans into source bytes, rewrite `/`-rooted internal links via configured prefix map.
- Golden fixtures: extract/merge round-trip on untouched input must be byte-identical (the "translate = identity" case).

### W2: `intl-ai-docs` binary (`crates/intl-ai-docs-cli`)

- `intl-ai-docs fill <src-dir> --locale <bcp47>` (+ `--dry-run`, `--stale`, `--regenerate`, `--keys` selector analog on SegIds/paths, `--no-cache`).
- `intl-ai-docs check <src-dir> --locale <bcp47>` (+ `--fail-on`).
- `intl-ai-docs.toml`: `[docs]` source_dir/targets/format/flavor, `links.prefix_map`, `frontmatter.keys`, provider/glossary/quality sections mirroring `intl-ai.toml`; `experimental = true` required or every command exits with a pointer.
- Tree walk: mirror `source_dir` -> `targets/<locale>/`; per-file `fill_flat` over that doc's segments; per-doc shard write.
- Progress events reuse `ProgressEvent`; add `Document` scope where the pipeline needs it (report key = `relpath#segid`).

### W3: docs checks (intl-ai-checks)

- `protected-region-parity` (built-in, always on): every masked token present exactly once in translated output; block skeleton untouched.
- `link-prefix`: internal links carry the target locale prefix; externals untouched.
- `corruption-signature`: raw `\n`/quote escapes, orphan braces, directive names translated, mid-line markdown starts.
- Reuse `judge`/`icu` on prose segments for free.

### W4: release + packaging (issue #107)

- Verify `dist plan` emits `intl-ai-docs` artifacts in the same release; fix dist config as needed.
- `packages/docs` -> `@intl-ai/docs` shim; changesets `fixed` group; trusted-publisher registration.
- Docs site page (marked experimental) + README mention.

### W5: self-consumption gate

- Regenerate `docs/es/` from `docs/` on a clean tree; `pnpm docs:build` + `docmd validate` zero warnings.
- `check` on the generated tree: zero corruption signatures; link report clean.
- Keep `adopt` (#108) as the follow-up for importing the current tree rather than regenerating.

## Sequencing

1. W0 refactor (foundation; isolatable PR).
2. W1 + W2 together: crate + CLI against replay-provider fixtures (no real provider needed until the end).
3. W3 checks land inside W2 PRs or as a fast follow.
4. W4 only after W1-W3 merge (verify dist shape early in parallel, it is the pipeline risk).
5. W5 dogfood as the merge gate for experimental -> the `docs/es` regeneration PR is the demo.

Suggested PR split: (1) `refactor(core): extract fill_flat/check_flat engine`, (2) `feat(docs): DocFormat + mdx extract/merge`, (3) `feat(docs): intl-ai-docs binary + config + lockfile`, (4) `feat(docs): checks (parity/link/corruption)`, (5) `feat(release): dist + @intl-ai/docs shim`, (6) `docs: regenerate docs/es via intl-ai-docs`.

## Test strategy

- Unit: mask/restore parity, SegId stability under reorder, frontmatter allowlist, link prefix rules, container boundary detection, unknown-directive warnings.
- Golden files: `tests/fixtures/mdx/**` round-trip; byte-identical on identity translations; preserved-region diff on translated output.
- E2E: replay provider (recorded responses) drives `fill`/`check` on a fixture tree; assert lockfile semantics (skip, stale, human-adoption).
- Corruption regression: fixture inputs reproducing the `docs/es` failure modes; `check` must flag every one.
- `cargo check --workspace`/`clippy`/`test` on every crate; wasm/uniffi keep building (DocFormat not exposed there yet).

## Open decisions (start of W1)

- Mask token form: `⟦0⟧` vs `<x id="0"/>` (pick whichever survives provider round-trips better; fixture-test both).
- Segment context hints in prompt (neighbor paragraph) — default off in v1, config-gated.
- Flavor flags: single `mdx` format with sub-options vs `markdown`/`mdx`/`docmd` flavors — decide from real divergence found while implementing.
