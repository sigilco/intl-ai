# Contributing to intl-ai

This doc covers the development workflow. Package-level specifics live in
`AGENTS.md` files; architecture lives in `ARCHITECTURE.md`.

## Quick start

```bash
cargo build            # build the workspace
cargo test             # run all Rust tests
pnpm install           # install shim deps
pnpm lint              # oxlint + oxfmt checks (run before pushing)
pnpm typecheck
pnpm test              # shim vitest suites
```

## Branches

- `main` — release snapshots only
- `develop` — integration; PRs target here
- `feature/<kebab>` / `fix/<kebab>` / `chore/<kebab>` — from `develop`, merge back

## Commits

Conventional Commits, imperative mood, subject ≤50 chars:

```
<type>(<scope>): <subject>
```

Types: `feat` · `fix` · `docs` · `test` · `chore` · `ci` · `refactor` ·
`build` · `perf`. Footer: `Closes #N`.

## Pull requests

- Run `cargo test --workspace`, `pnpm lint`, and `pnpm typecheck` first.
- Fill in the PR template; link the issue.
- Prefer small, single-purpose PRs over sweeping ones.
- Docs changes follow the writing rules in `AGENTS.md` (no em/en-dashes,
  no emoji, sentence-case headings).

## Releasing

Versions bump in lockstep: `workspace.package.version` in `Cargo.toml` is
the single source of truth for the binary and the `intl-ai` npm wrapper;
`@intl-ai/*` shims bump together via the changesets `fixed` group.

Release flow:

1. Bump `Cargo.toml` workspace version (and shim versions if they moved).
2. Update `CHANGELOG.md`: `scripts/changelog.sh update v<prev> HEAD <new>`
   and review the generated section.
3. Merge to `develop`, tag `vX.Y.Z`, push the tag. `release.yml` builds all
   platform artifacts, creates the GitHub Release, and publishes brew +
   npm. The release fails fast in `plan` if the CHANGELOG section is
   missing, so tag only after it merges.
4. Land the `develop -> main` snapshot commit on `main`.

## Getting help

Open an issue, or check the backlog board:
https://github.com/orgs/sigilco/projects/2

## Code of conduct

See `CODE_OF_CONDUCT.md`.
