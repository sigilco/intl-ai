# Architecture

intl-ai is a build-time i18n translation system. It runs entirely during the
build step, translates missing or stale JSON/YAML locale keys using any AI
provider, and writes results back to disk before the bundler emits its
output. Zero runtime overhead, zero coupling to the host app's i18n library.

For product context and roadmap see `.agents/docs/prd.md`.

---

## Components

A single Rust workspace under `crates/` produces the `intl-ai` binary. npm
packages under `packages/` are thin shims that spawn it.

| Crate               | Role                                                                                                                                             |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `intl-ai-core`      | Config resolution, locale model, lockfile, fill/check pipelines, `Progress` observer                                                             |
| `intl-ai-formats`   | `Format` trait + `FormatRegistry`: builtin json/yaml, `[[formats]]` exec plugins                                                                 |
| `intl-ai-exec`      | Subprocess protocol shared by exec checks and format plugins                                                                                     |
| `intl-ai-providers` | `Transport` implementations: http (OpenAI-compatible), command (local agents), replay (cassettes)                                                |
| `intl-ai-checks`    | `Check` implementations: icu, placeholder-parity, dialect, spec, exec, judge + quality gate                                                      |
| `intl-ai` (cli)     | clap surface: `fill`, `check`, `status`, `init`, `config`, `lockfile`, `spec`, `migrate`                                                         |
| `intl-ai-uniffi`    | UniFFI bindings: `IntlAi` object, `fill`/`check`/`status`, `fillAsync`/`checkAsync` + `IntlAiProgress` (cdylib/staticlib, not released via dist) |
| `intl-ai-wasm`      | wasm-bindgen exports for browser use: pure leaf functions + `runFill`/`runCheck` drivers                                                         |

| Shim                | Role                                                         |
| ------------------- | ------------------------------------------------------------ |
| `intl-ai` (npm)     | Platform-binary wrapper so `npx intl-ai` works               |
| `@intl-ai/unplugin` | Bundler plugin (vite, webpack, rollup, esbuild, rspack, bun) |
| `@intl-ai/next`     | `withIntlAi()` Next.js config wrapper                        |
| `@intl-ai/expo`     | Expo config plugin                                           |

Native packaging lives outside the npm tree: `swift/IntlAi` (SwiftPM +
XCFramework) and `kotlin/{intlai,intlai-android}` (JVM JNA JAR + Android AAR).

---

## Data flow (fill)

```
intl-ai.toml ──> ResolvedConfig (core)
                     │
   locale files ──> FormatRegistry.read (formats/exec plugin)
                     │
   lockfile shards (intl-ai.lock.d/) ──> diff: missing | stale | human-edited
                     │
   batches ──> Transport.fill (http | command | replay)
                     │
   Gate: [[checks]] validators + [quality] bands ──> per-key decision
                     │
   FormatRegistry.write + lockfile update + FillReport
                     └── ProgressEvents along the way (CLI --progress,
                         UniFFI callbacks, wasm onEvent)
```

`check` runs the same diff and gate read-only and reports findings.

---

## Trust boundaries

- Arbitrary programs run only when config asks: `command` providers,
  `exec` checks, `[[formats]]` plugins. Exec surfaces are honored from the
  project-root config only, never nested configs.
- Secrets interpolate via `${env:VAR}` and never land in lockfiles or
  reports.
- The fill gate, not the model, is the quality boundary: judge and rule
  checks decide per key what gets written.
