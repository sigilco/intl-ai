# intl-ai — Agent Context

AI-powered build-time i18n translation plugin. Hooks into any bundler via unplugin, translates JSON/YAML locale files at build time using any AI model (Vercel AI SDK). Zero runtime overhead, zero vendor lock-in. See `docs/prd.md` for product context and roadmap.

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

| Package                                                 | npm name                  | Purpose                                                                           |
| ------------------------------------------------------- | ------------------------- | --------------------------------------------------------------------------------- |
| `packages/api`                                          | `@intl-ai/api`            | Runtime-agnostic core — `runFill`, `IntlAiConfig`, JSON schema                    |
| `packages/unplugin`                                     | `@intl-ai/unplugin`       | Universal bundler plugin via unplugin 3 (Vite/Rollup/Webpack/esbuild/Rspack/etc.) |
| `packages/next`                                         | `@intl-ai/next`           | Next.js `withIntlAi()` wrapper — webpack plugin + Turbopack loader                |
| `packages/cli`                                          | `@intl-ai/cli`            | CLI: `intl-ai fill` and `intl-ai check`                                           |
| `packages/typescript-config`                            | `@repo/typescript-config` | Shared tsconfig — internal only, not published                                    |
| `examples/{next,legacy-next,vite,webpack,expo,flutter}` | —                         | Reference consumer apps, not published                                            |

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

### Config Files

Users place `intl-ai.config.ts` or `intl-ai.config.json` at project root. TypeScript is loaded via `jiti`; JSON is read and validated against the published JSON Schema. These are the only supported filenames; the legacy `.js` / `.mjs` / `.cjs` / `.intl-airc` filenames were removed in v0.2.0.

### Lockfile

`intl-ai.lock.json` in `localeDir`. Tracks per-key `sourceHash` (SHA-1), translation, and `origin: "ai"|"human"`. Never delete manually — drives staleness detection.

---

## GitHub Project Backlog

URL: https://github.com/users/sigilco/projects/10
Full policy (task fields, refinement checklist, `ghx` usage): `.agents/backlog-policy.md`
Product context and roadmap: `docs/prd.md`

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
