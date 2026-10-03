# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-10-03

### Added

- package svelte studio as the /demo/ page (#105)
- add jvm and android packages (#94)
- SwiftPM package + async fill/check to develop (#98)
- add intl-ai-wasm crate and browser demo (#91)
- add intl-ai-uniffi bindings crate (#90)
- pipeline progress observer hook (#95)
- pluggable locale formats via exec plugins (#92)

### Removed

- drop migrate subcommand; migrations are docs-only (#104)

### Documentation

- migrate site references to intl-ai.illo.fyi (#103)
- point backlog at org project board (#100)
- 0.4.x to 1.x migration guide (#89)

### Maintenance

- v0.7.0 release hygiene (changelog, docs, release notes) (#101)

## [0.6.0] - 2026-09-30

### Added

- config-plugin shim spawning the intl-ai binary (#75)
- phase-eval binary shim via withIntlAi (#73)
- binary-shelling shim + fill --judge-threshold (#72)
- quality bands + judge threshold (#70)

### Documentation

- oxfmt agents workspace table (#78)
- sweep reference + recipe guides for rust cli (#76)
- refresh readmes, apache-2.0 license, install.sh (#69)
- migrate vitepress to docmd (#71)

### Maintenance

- bump workspace to 0.6.0 (#84)
- npm trusted publishing + publish-shims job (#85)
- lockstep versioning across workspace (#74)
- intl-ai v0.5.0 — rust rewrite

## [0.5.0] - 2026-09-26

### Added

- shift-left validation gate (#65)
- incremental findings cache (#64)
- W2 framework, builtins, specs, exec (#61)
- yaml locales, stat-cache, lockfile merge, schema (#60)
- http + command transports, batching, prompts (#58)
- W0 rust core — workspace, config, replay, lockfile, cli (#56)
- default concurrency to 1 for agent-kind configs
- add kind discriminated union for agent config
- add command transport and agent CLI presets
- dispatch translateBatch through AITransport
- add AITransport port
- add check --dialect for British/American mismatches
- add per-locale style instructions

### Fixed

- gate process-group kill to unix for windows builds (#68)
- report contract + misc hardening (#63)
- W2b data-flow hardening + trust boundary (#62)
- send crush's prompt via argv, not stdin
- configure logtape meta logger level

### Documentation

- add VHS terminal demo recording

### Maintenance

- W3 cargo-dist pipeline + rust ci job (#67)
- regenerate json schema + oxfmt drift fix (#57)
- guard flat dot-notation key hashing in runFill (#46)
- add live integration test infra for AITransport
- guard against JSON schema drift
- bump versions for v0.4.1
- accept $schema in JSON config

## [0.4.1] - 2026-07-01

### Added

- replace citty+consola with cleye+logtape
- add quality build option passthrough
- add quality build option
- add quality block to JSON config and regenerate schema
- quality-aware runFill with retry loop integration
- default adversarial judge and quality retry loop
- add quality scoring types and lockfile schema extension
- rename model to provider, add modelParams

### Fixed

- z.url() replaces deprecated z.string().url()

### Changed

- tackle cross-cutting risks — lean base + batched extras
- consolidate config loading and tighten public surface
- extract runCheck into services/check
- restructure to Hexagonal (Ports and Adapters) architecture

### Documentation

- add clean URLs, sitemap, head tags, robots.txt, and fix npm bugs fields
- refresh guide headers and update provider/ai-model docs
- add observability and providers guide pages
- restructure i18n-libraries guide into directory
- add root ARCHITECTURE.md with system diagrams
- relocate prd to .agents/docs and refresh for v0.3
- document Hexagonal architecture in AGENTS.md

### Maintenance

- bump versions for v0.4.1
- accept $schema in JSON config
- thread JSON config model field to batch translators
- drop unimplemented docs:i18n:build from predocs hook
- sync lockfile after gray-matter removal
- format batch.test.ts
- harden repo for public launch
- TS6 hardening — target ES2022, verbatimModuleSyntax, treeshake
- route build error through logtape
- add pnpm catalog + pull examples into workspace
- pin Node 22 via mise, .node-version, engine-strict
- add dependency upgrade standardization plan
- add changeset for quality-aware fill loop
- add quality loop, judge, and fill tests
- update configs to use provider field
- add oxlint guard for hexagonal dependency direction

## [0.4.0] - 2026-06-30

### Added

- replace citty+consola with cleye+logtape
- add quality build option passthrough
- add quality build option
- add quality block to JSON config and regenerate schema
- quality-aware runFill with retry loop integration
- default adversarial judge and quality retry loop
- add quality scoring types and lockfile schema extension
- rename model to provider, add modelParams

### Fixed

- z.url() replaces deprecated z.string().url()
- write rendered formula to /tmp to avoid missing Formula/ dir

### Changed

- tackle cross-cutting risks — lean base + batched extras
- consolidate config loading and tighten public surface
- extract runCheck into services/check
- restructure to Hexagonal (Ports and Adapters) architecture

### Documentation

- add clean URLs, sitemap, head tags, robots.txt, and fix npm bugs fields
- refresh guide headers and update provider/ai-model docs
- add observability and providers guide pages
- restructure i18n-libraries guide into directory
- add root ARCHITECTURE.md with system diagrams
- relocate prd to .agents/docs and refresh for v0.3
- document Hexagonal architecture in AGENTS.md

### Maintenance

- version packages for v0.4.0
- drop unimplemented docs:i18n:build from predocs hook
- sync lockfile after gray-matter removal
- format batch.test.ts
- harden repo for public launch
- TS6 hardening — target ES2022, verbatimModuleSyntax, treeshake
- route build error through logtape
- add pnpm catalog + pull examples into workspace
- pin Node 22 via mise, .node-version, engine-strict
- add dependency upgrade standardization plan
- add changeset for quality-aware fill loop
- add quality loop, judge, and fill tests
- update configs to use provider field
- add oxlint guard for hexagonal dependency direction
- bump pnpm to v11.5.0

## [0.3.0] - 2026-06-25

### Added

- add LLMs dropdown menu in navbar
- add bun install option to all install code groups
- centralize URLs in VitePress via <DocsUrl> global component

### Fixed

- read pnpm version from packageManager field
- use pnpm with public-hoist-pattern to resolve workspace deps for bun build
- update contributing deployment section to Cloudflare Pages
- use intl-ai.pages.dev install script URL
- install dependencies before bun build in build-binaries job
- escape {{var}} placeholders in i18n-libraries.md
- update config file glob to \{ts,json\} (drop .js/.intl-airc)
- correct espetro → sigilco GitHub org references
- correct `{{variable}}` rendering and broken .html links
- add registry flag to npm publish command
- remove local Verdaccio registry from publishConfig
- align pnpm version in CI with packageManager

### Changed

- narrow @intl-ai/next to Turbopack bridge, delegate webpack to @intl-ai/unplugin

### Documentation

- update version refs and clarify brew/mise availability
- trim getting-started bundler examples to vite and webpack
- repurpose vue-i18n and i18next pages as library-only guides
- add build-systems section and restructure sidebar
- add i18n libraries compatibility page with ICU support table
- correct AGENTS.md file references (webpack-plugin, turbopack-loader)

### Maintenance

- rename homebrew tap to sigilco/homebrew-tap-intl-ai
- v0.3.0
- bump Homebrew tap formula on release
- add Homebrew formula template and mise registry entry
- migrate intl-ai.illo.fyi → intl-ai.pages.dev
- v0.1.0

## [0.2.0] - 2026-06-23

### Added

- inline loadConfig, drop @intl-ai/core dependency
- inline loadConfig with .ts and .json support

### Fixed

- add NPM_TOKEN alongside NODE_AUTH_TOKEN for changesets
- resolve workspace:\* deps in published packages

### Changed

- rename unplugin-intl-ai to @intl-ai/unplugin across codebase

### Documentation

- remove migration link from VitePress sidebar
- audit and fix stale docs
- add internals page, update sidebar, add runtime-agnostic changeset
- add logo assets
- consolidate guides, add llms plugin and logo
- add README files to published packages

### Maintenance

- v0.2.0
- remove @intl-ai/core workspace package
- apply formatting fixes
- switch config validation from ajv to zod
- deprecate @intl-ai/core with v1.0 removal notice
- add references and lean policy
- update dev dependencies to latest versions

## [0.1.0]

### @intl-ai/core

#### Minor Changes

- 0967775: Initial v0.1.0 release

### @intl-ai/unplugin

#### Minor Changes

- 0967775: Initial v0.1.0 release

#### Patch Changes

- Updated dependencies [0967775]
  - @intl-ai/core@0.1.0

### @intl-ai/next

#### Minor Changes

- 0967775: Initial v0.1.0 release

#### Patch Changes

- Updated dependencies [0967775]
  - @intl-ai/core@0.1.0

### @intl-ai/cli

#### Minor Changes

- 0967775: Initial v0.1.0 release

#### Patch Changes

- Updated dependencies [0967775]
  - @intl-ai/core@0.1.0
