# intl-ai-uniffi

UniFFI bindings for the intl-ai engine. Exposes `fill`, `check`, and `status`
to Swift, Kotlin, and any other language UniFFI generates, so native apps can
run translations without the CLI.

This crate is phase 1 of the native integration layer: it defines the API
surface and emits bindings. The SwiftPM package in `swift/IntlAi` (phase 2)
vendors the Swift bindings and wraps the compiled library in an XCFramework;
Android packaging is a later phase. This crate is excluded from the
cargo-dist release (`[package.metadata.dist] dist = false`) and produces a
`cdylib`/`staticlib`, not a binary.

## Exported surface

### `IntlAi` (object)

One resolved configuration. Immutable and thread safe.

| Constructor                                                                          | Behavior                                                                                                                                                                                     |
| ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `IntlAi(configPath: String?, workingDir: String?)`                                   | Loads `intl-ai.{toml,json,yaml}` from `configPath`, or discovers it under `workingDir` (default: process cwd). Relative paths inside the config resolve against the config file's directory. |
| `IntlAi.fromConfigString(config: String, format: ConfigFormat, workingDir: String?)` | Parses an inline TOML/JSON/YAML config. Paths resolve against `workingDir`. `extends` is rejected for inline configs.                                                                        |

| Method                             | Returns          | Notes                                                                                                                                             |
| ---------------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fill(options: FillOptions)`       | `FillReport`     | Translates missing (or scoped stale/regenerated) keys. Writes locale files and lockfile shards unless `dryRun`. Same semantics as `intl-ai fill`. |
| `fillJson(options: FillOptions)`   | `String`         | Same run; returns the exact JSON the CLI emits with `--format json`.                                                                              |
| `check(options: CheckOptions)`     | `CheckReport`    | Findings without writes. Same semantics as `intl-ai check`.                                                                                       |
| `checkJson(options: CheckOptions)` | `String`         | Same run; CLI-shaped JSON.                                                                                                                        |
| `status(locales: [String])`        | `[LocaleStatus]` | Per-locale inventory counts (read-only; no checks, no transport). Empty `locales` = config targets.                                               |
| `statusJson(locales: [String])`    | `String`         | `{"locales": [...]}` like `intl-ai status --format json`.                                                                                         |
| `sourceLocale()`                   | `String`         | Configured source locale.                                                                                                                         |
| `targetLocales()`                  | `[String]`       | Configured target locales.                                                                                                                        |
| `localeDir()`                      | `String`         | Resolved locale directory.                                                                                                                        |

### Records

- `FillOptions`: `locales`, `keys`, `keysFile`, `stale`, `regenerate`,
  `includeHuman`, `dryRun`, `noCache`, `validate`, `noValidate`,
  `judgeThreshold`, `yes`. An unscoped `regenerate` without `yes` raises
  `IntlAiError.Validation`, matching the CLI's `--yes` guard.
- `CheckOptions`: `locales`, `keys`, `keysFile`, `origin` (`OriginFilter`),
  `failOn` (`[FindingKind]?`; absent = config `check.fail_on`, empty = gate
  cleared), `noCache`.
- `FillReport` / `LocaleFillReport` / `FillFailure`, `CheckReport` /
  `LocaleDiff` / `CheckFinding` / `QualityScore`, `LocaleStatus`: field-for-field
  mirrors of the core serde reports, with counts widened to `UInt64`.
- Enums: `ConfigFormat` (`toml`/`json`/`yaml`), `OriginFilter` (`ai`/`human`),
  `FindingKind`, `ErrorKind`.
- `IntlAiError`: `Config`, `Lockfile`, `Io`, `Format`, `Transport`,
  `Validation`, `Other`.

### Transport

Provider config stays in `intl-ai.toml`, unchanged from the CLI: an
OpenAI-compatible endpoint is `[provider] kind = "http"` with `model`,
`api_key`, `base_url`. The core `Transport` trait is synchronous (blocking
HTTP, subprocess commands, replay cassettes), so every exported call is
synchronous too. There is no async runtime in the pipeline; foreign callers
should dispatch `fill`/`check` off the main thread.

## Generating bindings

```sh
crates/intl-ai-uniffi/scripts/generate-bindings.sh
```

Builds `libintl_ai_uniffi` and runs `uniffi-bindgen` in `--library` mode,
emitting into `crates/intl-ai-uniffi/bindings/`:

- `intl_ai_uniffi.swift`, `intl_ai_uniffiFFI.h`, `intl_ai_uniffiFFI.modulemap`
- `uniffi/intl_ai_uniffi/intl_ai_uniffi.kt` (package `uniffi.intl_ai_uniffi`)

The committed output is generated; regenerate it after changing the exported
API. Install `swift-format`/`ktlint` for formatted output (warnings only
otherwise). Other languages work the same way: `uniffi-bindgen generate
--library <lib> --language <lang>`.

## Layout

- `src/lib.rs`: `IntlAi` object and run methods
- `src/types.rs`: UniFFI records/enums and core-type conversions
- `src/error.rs`: `IntlAiError`
- `src/bin/uniffi_bindgen.rs`: bindgen CLI (behind `--features cli`)
- `tests/smoke.rs`: golden-path test against the replay transport
