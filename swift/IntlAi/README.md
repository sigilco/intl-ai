# IntlAi

SwiftPM package wrapping `intl-ai` for iOS and macOS. Runs `fill`, `check`,
and `status` in process through the UniFFI bindings in
`crates/intl-ai-uniffi`, with no CLI and no shelling out.

## Layout

- `Sources/IntlAi/intl_ai_uniffi.swift`: generated UniFFI bindings, vendored
  from `crates/intl-ai-uniffi/bindings/` (regenerate with
  `crates/intl-ai-uniffi/scripts/generate-bindings.sh` after API changes).
- `IntlAiFFI.xcframework`: static-library XCFramework holding
  `libintl_ai_uniffi.a` plus the UniFFI FFI headers. Built locally by
  `scripts/build-xcframework.sh` and gitignored; release builds ship it as a
  versioned zip asset instead.
- `scripts/build-xcframework.sh`: compiles the Rust `staticlib` for every
  Apple target, `lipo`s the simulator and macOS slices, and assembles the
  XCFramework.

## Build the XCFramework

```sh
swift/IntlAi/scripts/build-xcframework.sh
```

Requires Rust (with the Apple targets, which the script installs via
`rustup target add`) and Xcode. Slices produced:

| Slice                        | Rust targets                                  |
| ---------------------------- | --------------------------------------------- |
| `ios-arm64`                  | `aarch64-apple-ios`                           |
| `ios-arm64_x86_64-simulator` | `aarch64-apple-ios-sim`, `x86_64-apple-ios`   |
| `macos-arm64_x86_64`         | `aarch64-apple-darwin`, `x86_64-apple-darwin` |

### Why a static library and not a dynamic framework

A `cdylib` dylib is smaller on disk (~8 MB vs ~55 MB per arch) but must be
embedded and re-signed inside every app bundle, and it is what UniFFI's
SwiftPM guide calls the harder path. A static archive is the standard UniFFI
distribution: the app linker dead-strips unused code, so the final binary
cost is identical, while the package stays a plain `binaryTarget` with no
embedding or signing work for consumers.

## Consume the package

The XCFramework is a build artifact, so the package is consumed two ways:

- **Local path**: clone the repo, run `scripts/build-xcframework.sh`, then
  add `swift/IntlAi` as a local package dependency in Xcode or in your own
  `Package.swift`:

  ```swift
  .package(path: "../intl-ai/swift/IntlAi")
  ```

  and add `"IntlAi"` to your target's dependencies.

- **Release asset** (planned for v0.7.0): the release pipeline uploads
  `IntlAiFFI.xcframework.zip` to the GitHub release and the tagged
  `Package.swift` swaps the local `binaryTarget(path:)` for
  `binaryTarget(url:checksum:)`, so consumers only need
  `.package(url: "https://github.com/sigilco/intl-ai", from: "0.7.0")`.

## Usage

```swift
import IntlAi

let intl = try IntlAi(configPath: "intl-ai.toml", workingDir: projectDir)
let report = try intl.check(options: CheckOptions(
    locales: [], keys: [], keysFile: nil, origin: nil, failOn: nil,
    noCache: false))
```

Or with an inline config:

```swift
let intl = try IntlAi.fromConfigString(
    config: tomlString, format: .toml, workingDir: projectDir)
```

All calls are synchronous and blocking (`fill` performs provider I/O), so
dispatch them off the main thread. Errors surface as `IntlAiError` with a
`kind` matching the CLI's error taxonomy. See
`crates/intl-ai-uniffi/README.md` for the full exported surface.

## Requirements

- iOS 13.0+ or macOS 11.0+ (deployment targets set in `Package.swift` and
  matched by the Rust build via `IPHONEOS_DEPLOYMENT_TARGET` /
  `MACOSX_DEPLOYMENT_TARGET` in the script).
- Rust toolchain and Xcode for building the XCFramework; neither is needed
  once the binary target is distributed via a release asset.
