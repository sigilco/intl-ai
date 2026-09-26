# ADR 0001: Markdown translation as a companion tool, not core

**Status:** Accepted
**Date:** 2026-06-30
**Deciders:** Project owner

## Context

intl-ai ships build-time translation for JSON and YAML locale files via four packages: `@intl-ai/api`, `@intl-ai/unplugin`, `@intl-ai/next`, and `@intl-ai/cli`. All four assume structured, typed inputs.

When translating this project's own documentation (`docs/` to `docs/es/`), the existing core was bypassed entirely: a one-off script invoked an AI model, wrote markdown directly, and produced a fully or partially corrupted tree. Many files in the prior `docs/es/` contained JSON-escaped string literals (`\n`, stray `"`), markdown that started mid-line, and locale-relative links that pointed at `/guide/...` instead of `/es/guide/...`. Several rounds of cleanup were needed.

This raises a real product question: should intl-ai ship a way to translate markdown files within its scope, and if so, where should that capability live?

## Prior attempts and what they taught us

Two attempts were made before this ADR. Both failed in characteristic ways. They are recorded here so the resulting design is honest about its constraints.

**Attempt 1: JSON-wrapped markdown via the existing setup.** Markdown content was encoded into a JSON payload, sent through the existing translation pipeline, and the response was decoded back into markdown. Drawbacks:

- The AI model failed to enforce the JSON schema. Responses returned malformed JSON, extra prose, or partial valid payloads.
- Glue code to encode and decode markdown through JSON was brittle and hid corruption inside string escapes.
- This is almost certainly how the corrupted `docs/es/` tree was produced in the first place.

**Attempt 2: Direct agent-harness translation.** An agent harness (Claude Code) was invoked with skills intended to guide it on directive preservation, frontmatter handling, and link rewriting. Drawbacks:

- Skills were not auto-loaded by the harness reliably.
- Output quality varied widely across runs and across harnesses; behavior was non-deterministic.
- Adding skill-loading correctness to the harness was out of scope and is not a stable contract.

**Implication for this ADR.** The shipped tool must not rely on either of these mechanisms:

- It must not assume the model will honor a JSON schema, because model behavior on schema enforcement is unreliable and untestable.
- It must not assume any agent harness, skill loader, or prompt-only convention will be present at runtime.
- Translation must happen on **prose chunks**, parsed structurally by the tool, sent to the model as plain strings, and reassembled deterministically. The boundaries (what gets translated, what is preserved) are hard-coded in parsers and post-processors, not in the prompt.

## Decision

Ship markdown translation as a **separate companion tool**, `@intl-ai/markdown`, that **reuses the shared core** (`@intl-ai/api` for AI provider setup, config schema, and translation primitives) and **adds a markdown-aware layer on top**. It is **not** a core feature and not part of `@intl-ai/api`.

## Rationale

### Why not core

The current core assumes structured inputs whose shape is predictable at build time: locale files, key paths, string values, plural shapes. Markdown is none of those things.

- **Different content shape.** Frontmatter, fenced code, inline code, link targets, image paths, Vitepress directives (`::: tip`, `:button`, custom components), callouts, and arbitrary HTML pass through the same file. Translating it requires parsing, not just prompt-and-write.
- **Different audience.** The core serves app developers shipping locale files. Markdown translation serves maintainers and docs teams. Both groups care about correctness, but the failure modes differ: a corrupted locale file breaks a running application; a corrupted doc file breaks the docs site.
- **Different reliability bar.** Locale files are typed, validated by the schema in `@intl-ai/api`, and the build fails loudly when malformed. Markdown output is hard to validate automatically; the trust boundary must be different (parser checks + lint + human review), and that machinery does not belong on the core hot path.
- **Different upgrade cycles.** Adding markdown-specific parsing, prompt templates, and post-processing to `@intl-ai/api` would couple two release cadences with very different stability needs. The core is `0.3.0` and consumer-facing; doc tooling is internal-flavor.
- **Different correctness signal.** The cleanup we just ran is direct evidence that passing markdown through a JSON-shaped pipeline produces corruption. Whatever ships has to be markdown-aware from line one.

### Why a companion tool, not a plugin

- A plugin hooks into a bundler at build time. Markdown translation does not need a bundler; it runs on demand against a docs tree.
- A companion CLI is the natural shape: a single command run by maintainers and CI, with predictable file in / file out semantics.
- If demand later justifies a build-time hook (e.g. a Vitepress plugin that regenerates locales on rebuild), it can be built on top of the same companion core without re-architecting.

### Why reuse `@intl-ai/api`

Three pieces of the existing core translate cleanly:

1. **AI provider setup.** Vercel AI SDK wiring, model selection, prompt scaffolding. Reused as-is.
2. **Config schema.** `IntlAiConfig` and the JSON schema in `@intl-ai/api`. Reused as-is.
3. **Source hashing / staleness.** The lockfile pattern from `runFill` (`sourceHash`, per-key origin) can be reused at the file or section level for incremental translation.

Everything else (frontmatter, code fences, directives, link rewriting, validation) is markdown-specific and lives in the companion.

## Scope of v1

- **Package name:** `@intl-ai/markdown` (companion).
- **Surface:** one CLI command, `intl-ai-docs fill <src-dir> --locale <bcp47>`, with a `--check` mode for CI.
- **Targets:** Vitepress markdown (with the small Vitepress component set this project uses).
- **Pipeline per file:**
  1. Parse frontmatter (YAML/TOML) and lift it aside unchanged.
  2. Detect fenced code blocks and inline code; lift aside unchanged.
  3. Detect Vitepress directives and lift aside unchanged.
  4. Translate prose chunks individually via Vercel AI SDK. Each chunk is a plain string; the model never receives a whole markdown blob, never receives JSON, and never sees the file boundary. The trust boundary is the parser, not the prompt.
  5. Splice translated chunks back together with frontmatter, code, and directives reinserted at their original positions.
  6. Rewrite internal links from the source locale prefix (`/guide/...`) to the target locale prefix (`/es/guide/...`). External links and image paths untouched.
  7. Lint output for orphan braces, untranslated directives, broken internal links, and unescaped JSON literals (the corruption signature we just saw).
  8. Write file with a `.lock.json` analogue tracking per-file origin and source hash, mirroring `runFill` semantics.
- **Out of scope for v1:**
  - Bundler / unplugin integration.
  - Lockfile interoperability with `@intl-ai/api` (separate lockfile namespace).
  - Translation memory, glossary propagation beyond what the prompt template supports.
  - Hosting or SaaS. Free and open source, same as the rest of the project.

## Risks

- **Markdown variety is unbounded.** Pin v1 to Vitepress plus a small known component set. Anything outside that gets a clear "unsupported directive" warning rather than silent corruption.
- **AI translation is still imperfect.** Ship `--check` and a diff-friendly output. Document that human review is expected.
- **Corruption can reappear.** The post-translate lint pass is the trust boundary, not the prompt. If lint flags a file, do not write it.
- **Link rewriting has false positives.** Conservative rules: only rewrite path-shaped links starting with `/`, only inside the markdown body, only when the source file's locale prefix is unambiguous. Same for anchors; Vitepress anchors are stable so this is safe.
- **No JSON schema enforcement, no agent-harness contract.** The pipeline must work without depending on a JSON-output model contract or on a harness loading skills or instructions at runtime. If a future capability needs either, scope that future capability explicitly and re-justify.

## Consequences

- A new package, `@intl-ai/markdown`, lands in `packages/markdown` with its own changelog and versioning.
- `@intl-ai/api` stays unchanged. Any shared abstractions surface through it cleanly.
- Future docs translations of this project's own `docs/` use `@intl-ai/markdown` instead of ad-hoc AI calls.
- Consumers who want markdown translation adopt the companion explicitly; it does not bloat the core or `@intl-ai/unplugin`.

## Alternatives considered

- **Add markdown support to `@intl-ai/api`.** Rejected: conflated scopes, harder validation, slower release cadence for the core.
- **Use an external tool (Crowdin, Zanata, DeepL API).** Rejected for v1: loses the "any AI model, no vendor lock-in" differentiator that the project is built on. Can be revisited if `@intl-ai/markdown` proves insufficient.
- **Ship a thin wrapper that reuses `runFill` over the whole markdown blob.** Rejected: this is almost exactly what produced the corrupted `docs/es/` we just spent four phases cleaning up.
