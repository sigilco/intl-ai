# intl-ai-uniffi

UniFFI bindings for the intl-ai engine. Exposes `fill`, `check`, and `status`
to Swift, Kotlin, and any other language UniFFI generates, so native apps can
run translations without the CLI.

This crate is phase 1 of the native integration layer: it defines the API
surface and emits bindings. The SwiftPM package in `swift/IntlAi` (phase 2)
vendors the Swift bindings and wraps the compiled library in an XCFramework;
the Kotlin/JVM and Android packaging lives in `kotlin/`
(`io.github.sigilco:intlai` JAR, `io.github.sigilco:intlai-android` AAR).
This crate is excluded from the cargo-dist release
(`[package.metadata.dist] dist = false`) and produces a `cdylib`/`staticlib`,
not a binary.

## Exported surface

### `IntlAi` (object)

One resolved configuration. Immutable and thread safe.

| Constructor                                                                          | Behavior                                                                                                                                                                                     |
| ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `IntlAi(configPath: String?, workingDir: String?)`                                   | Loads `intl-ai.{toml,json,yaml}` from `configPath`, or discovers it under `workingDir` (default: process cwd). Relative paths inside the config resolve against the config file's directory. |
| `IntlAi.fromConfigString(config: String, format: ConfigFormat, workingDir: String?)` | Parses an inline TOML/JSON/YAML config. Paths resolve against `workingDir`. `extends` is rejected for inline configs.                                                                        |

| Method                                                         | Returns          | Notes                                                                                                                                             |
| -------------------------------------------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fill(options: FillOptions)`                                   | `FillReport`     | Translates missing (or scoped stale/regenerated) keys. Writes locale files and lockfile shards unless `dryRun`. Same semantics as `intl-ai fill`. |
| `fillJson(options: FillOptions)`                               | `String`         | Same run; returns the exact JSON the CLI emits with `--format json`.                                                                              |
| `check(options: CheckOptions)`                                 | `CheckReport`    | Findings without writes. Same semantics as `intl-ai check`.                                                                                       |
| `checkJson(options: CheckOptions)`                             | `String`         | Same run; CLI-shaped JSON.                                                                                                                        |
| `fillAsync(options: FillOptions, observer: IntlAiProgress?)`   | `FillReport`     | `fill` on a worker thread; the observer receives every `ProgressEvent` in order. Awaitable, never blocks the caller's executor.                   |
| `checkAsync(options: CheckOptions, observer: IntlAiProgress?)` | `CheckReport`    | `check` with the same worker-thread/observer contract as `fillAsync`.                                                                             |
| `status(locales: [String])`                                    | `[LocaleStatus]` | Per-locale inventory counts (read-only; no checks, no transport). Empty `locales` = config targets.                                               |
| `statusJson(locales: [String])`                                | `String`         | `{"locales": [...]}` like `intl-ai status --format json`.                                                                                         |
| `sourceLocale()`                                               | `String`         | Configured source locale.                                                                                                                         |
| `targetLocales()`                                              | `[String]`       | Configured target locales.                                                                                                                        |
| `localeDir()`                                                  | `String`         | Resolved locale directory.                                                                                                                        |

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
  `FindingKind`, `ErrorKind`, `Pipeline` (`fill`/`check`), `KeyOutcome`,
  `ProgressEvent`.
- `IntlAiError`: `Config`, `Lockfile`, `Io`, `Format`, `Transport`,
  `Validation`, `Other`.

### `IntlAiProgress` (callback interface)

Implemented by the consumer (Swift `protocol`, Kotlin `interface`) and passed
to `fillAsync`/`checkAsync` as `observer`. `onEvent(event: ProgressEvent)`
fires once per event: `runStarted`, `batchStarted`, `batchFinished`,
`keyDone`, `finding`, `localeFinished`, `runFinished`. All payloads are owned
data mirroring `intl-ai-core`'s `ProgressEvent`.

Threading: `onEvent` is invoked on the pipeline's worker thread, in emission
order, never concurrently within one run, and never after `runFinished`.
Implementations must be thread safe and should return quickly, since the
pipeline waits on each callback.

### Transport

Provider config stays in `intl-ai.toml`, unchanged from the CLI: an
OpenAI-compatible endpoint is `[provider] kind = "http"` with `model`,
`api_key`, `base_url`. The core `Transport` trait is synchronous (blocking
HTTP, subprocess commands, replay cassettes). The plain `fill`/`check`/`status`
calls block the calling thread; `fillAsync`/`checkAsync` run the same pipeline
on a dedicated worker thread (`std::thread` plus a oneshot, since the crate has
no async runtime) so the returned future never blocks the foreign executor.

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
- `src/progress.rs`: `IntlAiProgress` callback, `ProgressEvent` enums, worker-thread bridge
- `src/error.rs`: `IntlAiError`
- `src/bin/uniffi_bindgen.rs`: bindgen CLI (behind `--features cli`)
- `tests/smoke.rs`: golden-path test against the replay transport
- `tests/progress.rs`: ordered-event and worker-thread assertions for the async bridge
