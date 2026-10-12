# Research: extending intl-ai to document formats (markdown tier-1)

Status: research for re-scoping issue #18 (feat(markdown): `@intl-ai/markdown` companion).
Date: 2026-10-09.

## What changed since the ADR

ADR 0001 assumed the TS stack (`@intl-ai/api`, Vercel AI SDK, `.lock.json`).
The v0.7.0 Rust rewrite changed the landscape the ticket was written against:

- `intl-ai-formats` is now a real extension point: `Format` trait + `FormatRegistry` + `[[formats]]` exec protocol (v1, JSONL).
- The corpus model is `serde_json::Value` (nested key -> string), flattened to `FlatMap` (`BTreeMap<String, String>`) before the pipeline touches it.
- `Transport::translate` takes `Vec<TranslationEntry { key, source }>` + glossary + locale_instruction + syntax_hint.
- `Check` implementations (`icu`, `placeholder-parity` family, `judge`, `dialect`, `spec`, `exec`) all consume `CheckItem { key, source, target }` batches.
- The fill-time validation `Gate` re-prompts rejected keys with reviewer feedback.
- `Progress` observer, shard lockfile (`intl-ai.lock.d/<locale>.toml`), stat cache, key selector: all operate on flat keys.
- This repo's own docs migrated from VitePress to docmd; the ticket's "Vitepress markdown" pin should widen to "container-directive markdown" (`::: info|warning`, `::: tabs` + `== tab`), which docmd and VitePress share.

Everything in the middle of the pipeline is already document-agnostic.
Only the two ends (file IO, corpus shape) are locale-file-shaped.

## Landscape

| Tool                                                                   | Approach                                                                                                                                                                                              | Takeaway for intl-ai                                                                                                                                                                   |
| ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| po4a (mquinson/po4a)                                                   | Extract translatable strings to PO, translate, re-inject into document structure. Per-format modules. Fallback to source text for untranslated segments.                                              | The canonical decoupling: structure owned by the tool, text owned by the translator. `po4a-gettextize` also shows how to salvage an existing translation, which we need for `docs/es`. |
| mdpo (mondeja/mdpo)                                                    | md2po extractor + po2md merger over CommonMark. Inline markup can be kept inside msgids or split.                                                                                                     | Proof that "segments as flat units" maps cleanly onto markdown. Its plaintext-vs-markup option is the inline-mask tradeoff.                                                            |
| GitLocalize                                                            | Parses files into trackable segments, diffs source vs target segment-by-segment, TM reuse across files.                                                                                               | Segment-level provenance and drift detection, the same job our lockfile does for keys.                                                                                                 |
| Azure co-op-translator                                                 | Whole-file MD + notebooks + images via LLMs; versioned artifacts, freshness tracking, link rewriting to the translated tree, re-chunk only failures, GH Action + MCP.                                 | The strongest AI-native reference: link rewriting and per-file freshness are first-class. Still trusts the model inside each chunk.                                                    |
| rockbenben/md-translator                                               | Tokenize frontmatter/code/LaTeX/links/JSX into placeholders, translate prose with placeholders, restore losslessly. 25+ engines. Context-aware mode ships neighboring paragraphs.                     | Validates the "masked inline markup" pattern and context hints for quality.                                                                                                            |
| AarenWang/llm-translate                                                | `DocumentBlock` records + `TranslationChunk` planning + protection engine (placeholder swap, restore, chunk-level structural validation) + bilingual/validation artifacts.                            | Closest structural match to the ADR pipeline; treats validation output as an artifact, like our `check`.                                                                               |
| Factory translate-docs.py                                              | Regex split frontmatter/fences, translate prose segments, regex frontmatter fields.                                                                                                                   | The naive approach in the wild; works but fragile, good "floor" for what a real parser buys.                                                                                           |
| Docusaurus i18n / VitePress / mkdocs-static-i18n / mdbook-i18n-helpers | Directory-mirrored whole files (Docusaurus, VitePress), file-suffix locales (mkdocs), or gettext extraction (mdbook). Docusaurus deliberately translates MDX whole "to preserve translation context". | The filesystem layout we must emit is already standardized: mirror tree per locale. Whole-doc prompting is the context-quality extreme of the chunking axis.                           |
| Crowdin/Lokalise/Phrase/Weblate                                        | CAT SaaS: XLIFF-interchange internally, TM + MT + review.                                                                                                                                             | Vendor lock-in the ADR already rejected; XLIFF remains a useful vocabulary (trans-unit, skeleton, segmentation).                                                                       |
| Okapi / XLIFF-EM best practice                                         | Filter = extract trans-units + keep skeleton + merge back. Warns against over-segmenting (sentence-level extraction cripples the roundtrip).                                                          | Design caution: keep segment granularity at block level (paragraph/heading/list item), not sentence level.                                                                             |

Reddit/HN sentiment on AI doc translation (consistent across threads): whole-file LLM rewrites break formatting at scale;
the accepted fix is parse structurally, translate only prose, protect everything else, and validate mechanically.
That is exactly the ADR's trust model, and it is now the mainstream pattern (all three active OSS implementations above converge on it).

## Design patterns distilled

1. **Extract / translate / merge.** Parser produces a skeleton plus an ordered segment list; translation only ever sees segment strings. po4a, mdpo, Okapi, GitLocalize all share this shape.
2. **Placeholder masking for inline markup.** Inside a prose segment, inline code, link targets, image paths, and component tags become `@@n@@`-style opaque tokens, restored after translation and verified by a parity check. md-translator and llm-translate both do this; it preserves sentence-level quality (vs splitting at every inline boundary).
3. **Span replacement, not re-serialization.** `pulldown-cmark`'s `into_offset_iter()` yields `(Event, Range<usize>)`: byte offsets into the source. Splicing translations into the original byte buffer makes protected regions byte-identical by construction, which is the acceptance criterion. AST re-serialization (comrak, markdown-rs/mdast) rewrites formatting; avoid for output.
4. **Content-addressed segments.** Gettext msgid = the source text itself; GitLocalize and neokapi use content-addressed units. A source-hash segment id gives reorder tolerance and free cross-file translation memory (identical paragraphs share a key).
5. **Document = file, locale = directory.** Every docs system mirrors the tree (`docs/` -> `docs/es/`), so the tool's unit of work is a file pair, not a locale file.
6. **Freshness via per-file plus per-segment hashes.** co-op-translator tracks source freshness per file; lockfile shards already give us per-key (here: per-segment) sourceHash.
7. **Validation as an artifact.** `--check` on the translated tree plus bilingual diff output for human review; llm-translate emits exactly this.

## Where markdown fits the current architecture

`fill`/`check` decompose as: file IO + `flatten` -> **FlatMap engine** (diff, select, chunk, transport, gate, lockfile, observer, checks) -> `set_nested` + format write.
Markdown slots in as a different IO adapter over the same engine: extract produces the source FlatMap (`seg-id -> masked text`), translate with the same `Transport`/`Gate`/`Check` stack, merge produces target bytes.
This is the hexagonal split the codebase already follows: `Format` is one inbound adapter family, documents are another.

### Option A: document as corpus (markdown inside `Format`)

Parse `x.md` into a `Value` like `{ "fm.title": "...", "seg.0003": "..." }`.
Fails: `serialize(value, path)` gets no source document to splice against, so the skeleton must live inside the corpus as data.
Any corpus leaf is a translatable string to the pipeline, so the skeleton needs a protected-key convention in core (new format trait surface, new fill/check logic).
Also loses ordering and inline-markup masking fidelity.
Rejected: it smuggles the document into a shape it does not have, and pushes docs logic into the shared hot path.

### Option B: `DocFormat` sibling trait + shared FlatMap engine (recommended)

New crate `intl-ai-docs` (or `formats` submodule) defining:

```rust
trait DocFormat {
    fn name(&self) -> &str;                      // "markdown"
    fn extensions(&self) -> Vec<&str>;           // md, mdx maybe
    fn extract(&self, text: &str, path: &Path) -> Result<Document>;
    fn merge(&self, doc: &Document, target: &FlatMap) -> Result<Vec<u8>>;
}

struct Document {
    skeleton: Skeleton,                          // opaque byte-exact template
    segments: BTreeMap<SegId, Segment>,          // seg-id -> masked source text
}
```

- `SegId`: `h:<blake3/hex>` of normalized source text (content-addressed), plus stored position for diagnostics; or `fm.<key>` for frontmatter leaves.
- `Segment { kind, text (masked), context (nearest heading, position) }`; context goes into the prompt via existing `locale_instruction`-style plumbing or an entry comment field.
- `Skeleton`: source bytes + replacement spans (pulldown-cmark offsets) so `merge` is pure byte splicing; protected regions cannot change.
- Core refactor: lift the middle of `fill_locale` into a `fill_flat(source: &FlatMap, target: &FlatMap, shard: &mut Shard, ...)` (names TBD); `check` already batches `CheckItem`s, so docs checks become `Check` impls: `protected-region-parity`, `link-prefix`, `corruption-signature` (orphan braces, raw `\n` literals, untranslated directives).
- Lockfile: per-document shard under `intl-ai.lock.d/docs/<locale>/<relpath>.toml`, entries keyed by seg-id; same `sourceHash`/`origin`/`reviewed`/`quality` semantics.
- Config: `[[docs]]` section (source_dir, targets, format, link_prefix map) or `docs.*`; reuse glossary, `locale_instruction`, `batch_size`, providers, `[quality]` thresholds unchanged.
- CLI: `intl-ai docs fill|check` subcommands (one binary; the ADR's separate `intl-ai-docs` binary is unnecessary now that the core is one Rust CLI).
- Inline masking: `⟦i⟧`/`<x i="n"/>` tokens for inline code, link destinations, image paths, component tags; `placeholder-parity` check guarantees round-trip. Block-level: paragraphs, headings, list items, table cells become segments; fenced code, `:::` containers' boundaries, HTML/MDX blocks are skeleton. Unknown components warn, never silently corrupt (ADR pin).

Pros: zero semantic compromise of the corpus model; all quality machinery (judge, gate, quality bands, observer events, caches, dry-run) works per segment out of the box; `docs/es` salvage = adoption pass, same as human-authored keys; exec doc-formats can ride the same protocol later (v2: extract/merge instead of read/write).
Cons: needs the `fill_flat` refactor (mechanical), a second registry, and a doc-aware lockfile namespace.

### Option C: exec-protocol only

Register markdown as a `[[formats]] exec` plugin today.
Fails for the same reason as A: protocol v1 speaks the corpus tree, so an external plugin cannot express skeleton + segments either.
Tier-1 markdown support argues for doing B natively and extending the exec protocol to doc ops in v0.9+ if community plugins appear.

## Recommendation

Option B. It is the ADR's architecture expressed in the new Rust core: parsers own structure, the model sees only plain-string segments, reassembly is deterministic byte splicing, and `check` is the trust boundary.
The issue (#18) should be re-scoped: drop the TS references, retarget from VitePress to directive-markdown (docmd first, VitePress compatible), keep the separate-binary framing but ground it in the Rust workspace (`intl-ai-docs` thin CLI + `@intl-ai/docs` shim, see ecosystem shape below), and update acceptance criteria to byte-identical protected regions via span splicing.

## Ecosystem shape (multi-binary, minimal main surface)

The vision is tiers of localizable surfaces sharing one engine:
`Format` for catalogs (key -> string, any i18n library), `DocFormat` for documents (markdown tier-1), later `SurfaceFormat`/`AssetFormat` for listings, SEO/OG meta, support content, visual assets.
Each tier is an adapter family over the same FlatMap engine, so a new surface is a trait impl plus a thin CLI, not a pipeline fork.

Concrete layout:

```text
crates/
  intl-ai-core        # shared engine: fill_flat/check_flat, transport, gate, lockfile, observer, checks
  intl-ai-formats     # Format: json/yaml builtins + exec v1
  intl-ai-docs        # DocFormat trait + Document/Skeleton/SegId types + markdown impl
  intl-ai-cli         # `intl-ai` binary: catalogs only, unchanged surface
  intl-ai-docs-cli    # `intl-ai-docs` binary: docs fill|check|adopt (thin adapter, ~200 lines)
packages/
  intl-ai             # existing npm wrapper (cargo-dist npm installer)
  @intl-ai/docs       # new npm shim spawning `intl-ai-docs` (same spawn pattern as runIntlAiPipeline)
  @intl-ai/{unplugin,next,expo}
```

- Config: `intl-ai-docs.toml` (separate file, separate trust boundary; `[[docs]]` inside `intl-ai.toml` rejected so `intl-ai` never learns about document trees).
- Lockfile: `intl-ai-docs.lock.d/<locale>/<relpath>.toml` shards; separate namespace per the ADR.
- cargo-dist: a second dist-enabled package yields its own archives/installers in the same `vX.Y.Z` release; verify the multi-binary layout with `dist plan` early (single biggest pipeline unknown).
- UniFFI/WASM: DocFormat objects are ordinary trait objects; `docs fill` can surface in bindings later without redesign.
- Exec protocol: keep v1 corpus-only; protocol v2 can add `extract`/`merge` doc ops when community DocFormats arrive (Markdoc, RST would be exec plugins first, native later if demand holds).

## ROI assessment: next 1-2 DocFormats after markdown

1. **MDX, as a markdown flavor, not a new format.** `flavor = "mdx"` on the markdown DocFormat (alongside `commonmark`, `gfm`, `docmd`). The opaque-block machinery already tolerates JSX elements and `import`/`export` statements; what MDX adds is a tolerated-components list and attribute protection. It unlocks the largest real-world docs surface (Docusaurus, Nextra, Astro/Starlight, Next.js docs) for near-zero marginal cost.
2. **AsciiDoc.** Real docs-as-code demand (Antora, Fedora, Spring, Quarkus, Kotlin docs) and a block-oriented grammar (sections, delimited blocks, `NOTE:` admonitions, `include::` directives) that maps onto the same container/skeleton model. Rust parser maturity is thin (`asciidoc-parser` is partial), but block-level line scanning plus a span-splicer covers the 80% case. Prefer over reStructuredText: RST still matters in Python/Sphinx, but its inline role/directive syntax is nastier, momentum is flatter, and no usable Rust parser exists.
3. **Deferred to the surfaces tier, not DocFormat:** Fastlane metadata (`metadata/<locale>/*.txt`, app store listings; near-trivial but audience-thin), HTML/OG meta extraction (belongs with SEO/support surfaces), and PO/ARB/strings.xml/resx, which stay `Format` tier-2 adapters already on the v0.8.0 plan.

## Open questions to settle in the ticket rewrite

1. Segment identity: content-hash keys (`h:...`) vs positional (`seg.NNNN`) vs hybrid. Hash gives TM reuse; positional is debuggabler.
2. Inline mask vocabulary: `⟦0⟧`, `<x id="0"/>`, or XLIFF-ish `<ph>`? Pick tokens unlikely to be mangled by models.
3. Frontmatter: allowlisted keys (`title`, `description`, `nav*`) vs all string scalars; TOML/JSON frontmatter support beyond YAML.
4. Link rewriting: only `/`-rooted paths with configured prefix map, or also relative `../` links? Anchors?
5. Context hints: neighboring-paragraph context in the prompt (md-translator style) as an option, default on or off?
6. Granularity: block-level only (XLIFF best practice) or optional sentence splitting for long paragraphs?
7. Bilingual/review artifact output for `docs check` (diff-friendly side-by-side like llm-translate's `bilingual.md`)?
8. MDX/JSX: opaque-block-only in tier-1, or a tolerated-components list with attribute protection?
9. Salvage path: `docs adopt`/`--adopt` for importing an existing `docs/es` tree (the po4a-gettextize use case we have ourselves).
10. Crate boundary: new `intl-ai-docs` crate + `docs` subcommand on `intl-ai`, or modules inside `intl-ai-formats`/`intl-ai-cli`?

## Sources

- po4a: po4a.org, github.com/mquinson/po4a
- mdpo: github.com/mondeja/mdpo, mondeja.github.io/mdpo
- GitLocalize docs/blog: docs.gitlocalize.com, blog.gitlocalize.com
- Azure co-op-translator: github.com/Azure/co-op-translator, azure.github.io/co-op-translator
- md-translator: github.com/rockbenben/md-translator
- llm-translate: github.com/AarenWang/llm-translate
- Factory translate-docs.py: github.com/Factory-AI/factory (.github/scripts)
- Docusaurus i18n: docusaurus.io/docs/i18n; VitePress i18n: vitepress.dev/guide/i18n
- XLIFF extraction/merge best practice (TAPICC WG3); Okapi XLIFF Filter docs
- Internal: crates/intl-ai-formats/src/lib.rs, exec.rs; crates/intl-ai-core/src/{fill,check,transport,flatten,lockfile}.rs; .agents/docs/adr/0001-markdown-companion-tool.md; issue #18
