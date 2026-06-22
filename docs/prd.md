---
title: Product Requirements Document
---

# Product Requirements Document: intl-ai

## Table of Contents

- [Vision](#vision)
- [Product Vision](#product-vision)
- [Technical Decisions](#technical-decisions)
- [Research Findings](#research-findings)
- [Success Metrics](#success-metrics)
- [0.1.0 Release Blockers](#010-release-blockers)
- [Scope Boundaries](#scope-boundaries)

## Vision

### Core Objective (One Sentence)

AI-powered build-time i18n translation plugin that works with any bundler and any AI model. Plug in, translate, ship.

### Product Positioning

**Swiss army knife (modular):** core plugin + CLI + management features, not a full platform

**Target Users:** Indie developers AND mid-size companies

**Core Differentiators:**

1. Build-time AI translation: zero runtime overhead, works with existing i18n libraries
2. Universal bundler support: one plugin via unplugin, works with every bundler
3. Model agnostic / privacy: use any AI model (local, self-hosted, or cloud), no vendor lock-in

## Product Vision

### Delivered (as of 0.1.0):

- **Core plugin:** `@intl-ai/core` AI translation pipeline
- **Bundler plugins:** unplugin for Vite, Webpack, Rollup, esbuild, Rspack
- **Next.js integration:** `@intl-ai/next`
- **CLI:** `packages/cli` with `fill` and `check` commands

### 12-Month Feature Roadmap (Modules in Scope):

1. ~~**CLI**~~: **DELIVERED**: `packages/cli` with `fill` + `check` commands
2. **Incremental/delta translation:** git diff based, only translate new/changed keys
3. **Translation quality scoring:** LLM-as-a-Judge (same model, different session, self-score 0-1 per key, flag low scores)

### 0.1.0 Release Scope:

**Minimal:** ship current state as-is (bundler plugin works, basic translation works)

Get it out, iterate from there.

### Monetization:

**Free / OSS only:** fully open source, no paid tiers, community-driven

## Technical Decisions

### i18n Library Support (All of These):

- vue-i18n (already documented)
- i18next (already documented)
- next-intl
- FormatJS / react-intl
- Paraglide
- Any JSON-based i18n setup (generic)

### Test Strategy:

**Tests-after** with existing vitest infrastructure (confirmed across all packages)

Agent-executed QA via Playwright for docs site

### Open Questions (Deferred, decide during implementation):

- Delta translation: git diff at key level (new keys + changed values). Deleted keys: TBD.
- Quality scoring: scoring prompt design, confidence threshold TBD.
- Paraglide: generic JSON support, no specific integration needed.

### Timeline:

- **0.1.0**: Release NOW, iterate fast
- **Post-0.1.0**: Monthly iterations

## Research Findings

**From codebase exploration:**

- **Monorepo**: pnpm workspaces + Turborepo + changesets
- **4 packages**: core, unplugin, next, cli + shared typescript-config
- **Docs**: VitePress site at `docs/` with existing guides (getting-started, ai-model, configuration, next-js, vue-i18n, i18next, contributing)
- **CHANGELOG**: empty (unreleased)
- **Version bumps**: to 0.1.0 already done locally (not committed)
- **JSR publishing**: removed from CI
- **Registry**: `localhost:4873` is Verdaccio local dev (expected, not a blocker)

## Success Metrics

### Primary: Adoption / Community Growth

- Repos using it
- Blog posts
- Word of mouth

### NOT Primarily

- Stars/downloads (vanity metrics)
- Feature milestones

## 0.1.0 Release Blockers (Must Complete Before Publish):

- [ ] Merge develop to main
- [ ] Publish to npm
- [ ] Docs site live (VitePress)
- [ ] Working examples (next, vite, webpack at minimum)

## Scope Boundaries

### INCLUDE:

- Build-time translation (bundler plugin)
- On-demand CLI translation (packages/cli, already delivered)
- Incremental/delta translation (git diff based)
- Translation quality scoring (LLM-as-a-Judge, same model, different session)
- i18n library support: vue-i18n, i18next, next-intl, FormatJS, Paraglide, any JSON-based
- Model agnosticism: any AI model via AI SDK compatible API
- Documentation: one guide per supported library, one guide per AI provider
- Privacy: no telemetry, no data collection

### EXCLUDE (Explicit Guardrails):

- No cloud dashboard / SaaS platform
- No paid tiers or license tiers
- No user accounts / authentication
- No translation memory hosted by the project (local git-based TM only)
- No automatic translation updates without human consent (human-in-the-loop)
- No CLI subcommands beyond `fill` and `check` (no serve, deploy, config wizard)
- No external evaluation service for quality scoring (same model, different session only)
- No support for binary i18n formats (PO, XLIFF) unless community-contributed
- No RTL-specific handling (out of scope; handled by user's i18n lib)
