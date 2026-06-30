# Dependency upgrade + tooling standardization

> **GitHub Issue**: #17
> **Project Item**: https://github.com/users/espetro/projects/10/items/PVTI_lAHOAZgqI84BZ1b3zgxP3u8

## Context

The working tree already carries most of the requested version bumps (uncommitted):
Node `>=22`, vitest `4.1.9`, tsdown `0.22.3` (tsup fully gone except dead scripts),
jiti `2.7.0`, zod `4.4.3` (and the code already uses zod 4 idioms:
`z.toJSONSchema`, `z.function().input().output()`, two-arg `z.record`),
TypeScript `6.0.3` in the published packages, and `@logtape/logtape 2.2.1`
already installed and wired into `@intl-ai/api`.

So this is not a "bump versions" job. The remaining work is the code-level
migrations the user asked for and, crucially, the standardization/enforcement
that stops versions drifting per package again. Concretely:

- CLI still runs on `citty` + `consola`; replace both with `cleye` (arg parsing)
  plus `logtape` (output).
- One stray `console.warn` in unplugin; route through `logtape`. `chalk` is not
  used anywhere (nothing to do).
- `@intl-ai/api` declares BOTH `jiti` and `tsx`; keep `jiti` (runtime config
  loader), drop `tsx` (only used by one build script).
- Engines/versions are hand-pinned per package and drift (expo lags, examples on
  TS 5.5.4). Introduce a pnpm catalog as the single source of truth and pull
  `examples/*` into the workspace so they are enforced too.
- Raise the minimum Node to the floor tsdown requires (`^22.18.0 || >=24`).

Decisions (from user):
catalog + examples in workspace; CLI output all through logtape (it has built-in
coloring); schema build via jiti CLI; keep unplugin peer ranges wide (bump only
example test deps).

Backlog note: per project policy this work needs a refined task on
https://github.com/users/sigilco/projects/10 before implementation. Create/refine
it (label `infra`, effort `L`) and copy this plan to
`.agents/plans/2026-06-30-dependency-upgrade-standardization.md`.

---

## 1. CLI: cleye replaces citty + consola

Files: `packages/cli/src/index.ts`, `packages/cli/src/commands/fill.ts`,
`packages/cli/src/commands/check.ts`, `packages/cli/package.json`.

- `index.ts`: replace citty `defineCommand`/`runMain` + lazy `subCommands` with
  cleye `cli({ name, version, commands: [...] })`. Pull `version` from
  package.json (currently hardcoded stale `"0.2.0"`) instead of inlining.
- `fill.ts` / `check.ts`: convert each `defineCommand({ args, run })` to a cleye
  `command({ name, flags, parameters }, callback)`. Map current args:
  `config` (string, default `intl-ai.config.json`), `locale` (string),
  `force`/`silent`/`dry-run` (boolean, default false). Note cleye uses camelCase
  flag keys (`dryRun`), so update the `args["dry-run"]` access sites.
- Replace every `consola.*` call (fill.ts: `start`/`success`/`info`/`fail`/
  `error` + `consola.level = 0`; check.ts: `error`/`warn`/`info`/`log`/`success`)
  with the logtape logger. Move the existing `configure(...)` bootstrap (fill.ts
  lines 42-45) into a tiny shared `src/logger.ts` so both commands reuse it.
  Configure `getConsoleSink()` with an ANSI color formatter
  (`getAnsiColorFormatter`) for the colored output consola provided; gate
  `lowestLevel` on `--silent` (silent => `"error"` only).
- The `\r` progress line + per-locale `process.stdout/stderr.write` calls in
  fill.ts stay as-is (they are deliberate inline terminal writes, not consola).
- `package.json`: remove `citty`, `consola`, redundant `jiti` (only api imports
  it); add `cleye`. Fix `dev` script `tsup --watch` -> `tsdown --watch`.
- Update `packages/cli/src/commands/*.test.ts` and `AGENTS.md` (CLI doc still
  references a nonexistent `src/utils/progress.ts` reporter; align to reality).

## 2. logtape replaces remaining console usage

- `packages/unplugin/src/index.ts:30`: replace `console.warn("[intl-ai] ...")`
  with a logtape logger (`getLogger(["intl-ai","unplugin"])`). Add
  `@logtape/logtape` to `packages/unplugin/package.json` deps.
- `packages/api/scripts/build-schema.ts` `console.log` x2: leave (build-time
  script, not library/runtime code) — or switch to `process.stdout.write` if we
  want zero `console.*`. Recommend leaving; oxlint `no-console` is `warn` and
  scripts are not linted as src.
- chalk: confirmed absent. No action.

## 3. jiti only (drop tsx)

- `packages/api/package.json`: remove `tsx` devDep; change
  `"schema:build": "tsx scripts/build-schema.ts"` -> `"jiti scripts/build-schema.ts"`.
- Remove redundant `jiti` runtime deps from `cli`, `next`, `unplugin`
  (only `packages/api/src/infrastructure/config/loader.ts` imports it; the others
  get it transitively via `@intl-ai/api`). Keep `jiti` in `api`.

## 4. Minimum Node 22 + unified engines

- Set `engines.node` to `"^22.18.0 || >=24"` (tsdown's floor) in every
  package.json: root, api, unplugin, cli, typescript-config, and the
  examples. Add it to `packages/next` and `examples/{vite,webpack,expo}` which
  currently lack `engines`.
- Add repo-root toolchain pins per global tooling policy (mise): `mise.toml` with
  `node = "22"` and `.node-version` = `22`. Add `.npmrc` with
  `engine-strict=true` so the engine floor is enforced on install.
- Pin `packageManager` exact (e.g. `pnpm@11.x.y`) at root.

## 5. Standardize/enforce versions: pnpm catalog + examples in workspace

This is the core "enforce for all" ask.

- `pnpm-workspace.yaml`: add `examples/*` to `packages:` so example apps become
  workspace members (they already use `workspace:*` for `@intl-ai/*`). Add a
  `catalog:` block holding the single shared version for each cross-package dep:
  `typescript`, `vitest`, `@vitest/coverage-v8`, `tsdown`, `@types/node`, `zod`,
  `jiti`, `@logtape/logtape`, `cleye`, `unplugin`, `react`, `react-dom`,
  `@types/react`, `@types/react-dom`, `next`. Keep `saveExact: true`.
- Rewrite each `package.json` to reference `catalog:` instead of literal versions
  for every dep present in the catalog. Example: `"typescript": "catalog:"`.
- This auto-fixes the existing drift: `examples/expo` tsdown `0.21.10`->catalog
  `0.22.3` and vitest `4.1.6`->`4.1.9`; examples TS `5.5.4`->`6.0.3`.
- React split: legacy-next/expo intentionally stay on React 18 — give them a
  named catalog (`catalog:react18`) vs the default React 19 catalog so both are
  still enforced rather than free-floating.
- Add a CI/`pretest` guard: `pnpm dedupe --check` (or `pnpm -r exec` lint) to
  catch any non-catalog version creeping back in.

## 6. Bump example test deps; keep unplugin peers wide

- `packages/unplugin/package.json`: leave peerDependencies ranges permissive
  (no breaking bump for consumers). No vite/next devDep needed there.
- Bump the actual versions consumed by example apps (the user's "vite/next too
  old" point lands here, since unplugin itself has no vite/next dep):
  - `examples/vite`: vite is already `8.0.12` (current) — just move to catalog.
  - `examples/next`: next `16.2.6` (current) — to catalog.
  - `examples/legacy-next`: intentionally next `15` / React 18 (the "legacy"
    fixture) — keep, but via the React-18 catalog.
  - `examples/webpack`: webpack `5.106.2` current — to catalog where applicable.
  - `examples/expo`: align tsdown/vitest/typescript via catalog (fixes the lag).

## 7. TypeScript 6 + tsconfig/tsdown/vitest hardening

- `packages/typescript-config/base.json`: add `"target": "ES2022"` (TS6 raises
  defaults; pin explicitly) and `"verbatimModuleSyntax": true` (codebase already
  uses `import type` consistently, pairs with existing `isolatedModules`).
- Add a `typescript` devDep (catalog) to `packages/unplugin` and root (both run
  `tsc`/typecheck but rely on hoisting today).
- Fix `packages/unplugin/tsconfig.json` to extend `@repo/typescript-config/base.json`
  (alias) instead of the relative `../typescript-config/base.json` path.
- Align tsdown configs: use `format: ["esm"]` consistently and add
  `treeshake: true` everywhere (only api has it now). Optionally extract a shared
  `defineConfig` factory in `@repo/typescript-config` or a root helper.
- Add `environment: "node"` to `packages/{cli,next}/vitest.config.ts` (api and
  unplugin already set it).

## 8. zod 4 nit (optional)

- `packages/api/src/schema/json-config.ts:20`: `z.string().url()` is deprecated
  in zod 4 -> `z.url()`. Re-run `pnpm --filter @intl-ai/api schema:build` after;
  regenerate `intl-ai.schema.json` + `docs/public/schema/v1.json`. Verify the
  emitted JSON Schema for `baseURL` is unchanged (still `format: "uri"`).

---

## Critical files

- CLI migration: `packages/cli/src/index.ts`, `packages/cli/src/commands/fill.ts`,
  `packages/cli/src/commands/check.ts`, new `packages/cli/src/logger.ts`,
  `packages/cli/package.json`, CLI `*.test.ts`, `packages/cli/AGENTS.md`.
- logtape: `packages/unplugin/src/index.ts`, `packages/unplugin/package.json`.
- Standardization: `pnpm-workspace.yaml`, every `package.json`, new `mise.toml`,
  `.node-version`, `.npmrc`.
- TS6/config: `packages/typescript-config/base.json`,
  `packages/unplugin/tsconfig.json`, all `tsdown.config.ts`, cli/next
  `vitest.config.ts`.
- zod nit + schema: `packages/api/src/schema/json-config.ts`, regenerated
  schema JSON files.

## Verification

1. `pnpm install` clean (catalog resolves; engine-strict passes on Node 22).
2. `pnpm -r exec node -e "process.exit(0)"` sanity; confirm one resolved version
   per catalog dep: `pnpm why typescript`, `pnpm why vitest`, `pnpm why zod`
   show a single version across the tree.
3. `pnpm build` (turbo) green for all packages.
4. `pnpm typecheck` green with `verbatimModuleSyntax` + `target` added.
5. `pnpm test` green; CLI command tests updated for cleye flag shape (camelCase
   `dryRun`).
6. `pnpm --filter @intl-ai/api schema:build` runs via jiti, emits identical
   schema (git diff on the two JSON files only shows the intended `z.url`
   format, if applied).
7. Manual CLI smoke: build `@intl-ai/cli`, run `intl-ai check` and
   `intl-ai fill --dry-run` against an example (`examples/next`); confirm cleye
   parses flags, `--silent` suppresses, and logtape output is colored.
8. `grep -rn "citty\|consola\|tsup\|\\btsx\\b\|console\\." packages/*/src` returns
   only intentional remainders (build-script console.log, inline progress writes).
9. Confirm no `engines`-less package remains:
   `grep -L '"node"' packages/*/package.json examples/*/package.json`.
