# @intl-ai/api — Agent Context

Runtime-agnostic translation core. Zero Node.js-only deps in the domain rings. See root `AGENTS.md` for workspace-wide context.

---

## Internal Architecture: Hexagonal (Ports and Adapters)

```
src/
  core/           Pure domain — zero IO, zero framework deps
    diff.ts         findMissingTranslations, flattenObject, lockfileEntryToMap
    hash.ts         hashSha1 (SHA-1 via Web Crypto, non-cryptographic fingerprinting)
    types.ts        Domain entities: ValidationResult, TranslationEntry, TranslationResult, etc.
  ports/          Interfaces only (the hexagon edges)
    provider.ts     AIProvider
    processor.ts    IntlAiProcessor
    format.ts       LocaleFormat
    hook.ts         TranslationHook
  services/       Use-cases as isolated feature slices
    fill/           runFill — translates missing/stale keys
    check/          runCheck — reports missing/stale keys without writing
  adapters/       Port implementations, grouped by capability
    providers/      openai.ts, anthropic.ts, registry.ts
    processors/     icu.ts, passthrough + createProcessor
    formats/        json.ts, yaml.ts (each implements LocaleFormat)
  infrastructure/ Host/runtime IO
    fs.ts           Node.js file system wrappers (readText, writeText, pathExists, ...)
    config/
      loader.ts     loadConfig (search from cwd), loadConfigFromPath (explicit path)
  lockfile/       LockfileManager + types (domain-adjacent, not pure)
  schema/         Zod schema (IntlAiJsonConfigSchema), JSON Schema file, jsonConfigToIntlAiConfig
  types.ts        IntlAiConfig + IntlAiConfigSchema (imports from ports/)
  index.ts        Public surface: runFill, runCheck, IntlAiConfig, IntlAiConfigSchema
  internal.ts     Explicit named re-exports for inter-package use (cli, unplugin, next)
```

### Dependency rule

`adapters/` and `infrastructure/` may import `core/` and `ports/`.
Each `services/<feature>/` imports `core/` and `ports/` only — never another slice.
`core/` and `ports/` import nothing from the outer rings.

Enforced by `oxlintrc.json` (no-restricted-imports overrides).

### Extending the system

**New AI provider**: implement `AIProvider` (ports/provider.ts), add to `adapters/providers/registry.ts`.

**New locale format**: implement `LocaleFormat` (ports/format.ts), add to `adapters/formats/`.

**New processor/validator**: implement `IntlAiProcessor` (ports/processor.ts), add to `adapters/processors/`.

**New use-case**: add a slice under `services/<feature>/` with its own `index.ts` barrel. Export through `internal.ts`.

### Forward compatibility note

The `core/` and `ports/` boundary is designed to become the Rust/TS language boundary in v1 (see root AGENTS.md and `.agents/plans/2026-06-27-hexagonal-architecture-api.md`).
Keep ports serializable (no closures crossing the seam) and keep `core/` free of JS-only runtime deps.
