# intl-ai — Agent Context

AI-powered build-time i18n translation CLI. A single Rust binary fills missing locale keys at build time via AI providers, validates ICU MessageFormat output, and tracks provenance in a sharded lockfile. npm packages under `@intl-ai/*` are thin shims that spawn the binary. Zero runtime overhead, zero vendor lock-in. See `.agents/docs/prd.md` for product context and roadmap.

---

## Tech Stack Cheatsheet

| Task                   | Command                                            |
| ---------------------- | -------------------------------------------------- |
| Install                | `pnpm install`                                     |
| Build all              | `pnpm build`                                       |
| Build one package      | `pnpm --filter @intl-ai/api build`                 |
| Test all               | `pnpm test`                                        |
| Test one package       | `pnpm --filter @intl-ai/api test`                  |
| Lint                   | `pnpm lint`                                        |
| Format                 | `pnpm format`                                      |
| Format check           | `pnpm format:check`                                |
| Typecheck              | `pnpm typecheck`                                   |
| Docs dev               | `pnpm docs:dev`                                    |
| Local registry publish | `pnpm publish:local` (Verdaccio at localhost:4873) |
| New changeset          | `pnpm changeset:add`                               |
| Release publish        | `pnpm release`                                     |

**Toolchain**: pnpm@11 · Turborepo 2 · tsdown · vitest 4 · oxlint · oxfmt
**Node requirement**: >=22.0.0

---

## Workspace Map

| Package                                                 | npm name                  | Purpose                                                                   |
| ------------------------------------------------------- | ------------------------- | ------------------------------------------------------------------------- |
| `crates/intl-ai-{core,formats,providers,checks,cli}`    | — (binary `intl-ai`)      | Rust workspace: config, locale formats, provider transports, checks, CLI  |
| `crates/intl-ai-uniffi`                                 | — (cdylib, not released)  | UniFFI bindings for Swift/Kotlin; excluded from cargo-dist                |
| `swift/IntlAi`                                          | — (SwiftPM, not on npm)   | SwiftPM package: vendored Swift bindings + XCFramework of the uniffi lib  |
| `packages/api` (removed)                                | `@intl-ai/api`            | TS era, deleted from the tree; npm stays deprecated at its last version   |
| `packages/cli` (removed)                                | `@intl-ai/cli`            | TS era, deleted from the tree; npm stays deprecated at its last version   |
| `packages/unplugin`                                     | `@intl-ai/unplugin`       | Bundler shim via unplugin 3 — spawns the `intl-ai` binary in `buildStart` |
| `packages/next`                                         | `@intl-ai/next`           | Next.js `withIntlAi()` config wrapper — spawns the binary at config eval  |
| `packages/expo`                                         | `@intl-ai/expo`           | Expo config plugin — spawns the `intl-ai` binary during prebuild          |
| `packages/typescript-config`                            | `@repo/typescript-config` | Shared tsconfig — internal only, not published                            |
| `examples/{next,legacy-next,vite,webpack,expo,flutter}` | —                         | Reference consumer apps, not published                                    |

---

## Key Conventions

### Commits

Conventional Commits: `<type>(<scope>): <subject>` — imperative, lowercase, ≤50 chars subject. Footer: `Closes #N`.
Types: `feat` · `fix` · `docs` · `test` · `chore` · `ci` · `refactor` · `build` · `perf`

### Branches

- `main` — production releases only
- `develop` — integration (PRs target here)
- `feature/<kebab>` · `fix/<kebab>` · `chore/<kebab>` — from `develop`, merge back to `develop`
- `release/vX.Y.Z` — from `develop`, merges to `main` + back-merge to `develop`

### Changeset Workflow

`pnpm changeset:add` → `pnpm changeset:version` → `pnpm release` (CI-driven on `main`)

### Versioning

All published artifacts share one version, currently the `workspace.package.version` in `Cargo.toml`:

- Rust crates pin `version.workspace = true`; the binary, GitHub Release, brew formula, and the `intl-ai` npm wrapper all inherit it via cargo-dist (tag `vX.Y.Z` to release).
- `@intl-ai/*` npm shims bump in lockstep via the changesets `fixed` group in `.changeset/config.json`. Add new shims to that group.
- Deprecated `@intl-ai/{api,cli}` were deleted from the tree at their last released versions (0.4.1 and 0.3.2); npm stays deprecated and they never republish.

### Config Files

Users place `intl-ai.toml` (or `.json`/`.yaml`) at project root. Full schema: `docs/guide/configuration.md`.

### Lockfile

`intl-ai.lock.d/<locale>.toml` shards. Tracks per-key `sourceHash`, translation, `origin: "ai"|"human"`, `reviewed`, `quality`. Never delete manually — drives staleness detection, the check cache, and the fill gate.

---

## GitHub Project Backlog

URL: https://github.com/users/sigilco/projects/10
Full policy (task fields, refinement checklist, `ghx` usage): `.agents/backlog-policy.md`
Product context and roadmap: `.agents/docs/prd.md`

---

## Agent Harness Notes

- Plans go to `.agents/plans/<YYYY-MM-DD>-<purpose>.md` (per global policy)
- `.omo/` is the Omo framework's internal state — separate from Claude Code plans
- `CLAUDE.md` at every level is a symlink to its sibling `AGENTS.md` — do not edit `CLAUDE.md` directly
- Package-level `AGENTS.md` files contain only what's specific to that package; don't repeat root-level context

---

## Writing style

These rules apply to docs, plans, and README files:

- **No em-dashes (`--`) or en-dashes (`--`) in prose.** Use commas, colons, semicolons, parentheses, or split into two sentences.
- **No specific AI model names that will rot.** Refer to providers, not model versions. The only exception is a stable OpenRouter free-model pointer with a "free at time of writing" note.
- **User-focused, not implementation-focused.** Docs answer "how do I use this?" Implementation details live in `docs/internals.md` or code comments.
- **No emoji.**
- **Sentence case for headings** in user-facing docs. Title Case only for code identifiers, table headers, and config keys.
- **One sentence per line** in plan files to make diffs easier.
