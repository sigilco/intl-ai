---
title: SwiftUI
description: AI translation for SwiftUI i18n via the IntlAi SwiftPM package or an Xcode build phase. Translations happen at build time.
---

# SwiftUI

You can integrate `intl-ai` into a SwiftUI project in two ways. The `IntlAi` SwiftPM package embeds the engine so app or tooling code can call `fill`, `check`, and `status` in process, while the CLI runs as an Xcode build phase for pure build-time translation with no runtime API.

::: tabs

== tab "SwiftPM package"

The `IntlAi` package lives at `swift/IntlAi` in the intl-ai repo and wraps the Rust engine in a static XCFramework for iOS and macOS. Until the v0.7.0 release asset is published, build the XCFramework once and depend on the package by local path:

```sh
git clone https://github.com/sigilco/intl-ai
intl-ai/swift/IntlAi/scripts/build-xcframework.sh
```

Then add `swift/IntlAi` as a local package in Xcode (**File > Add Package Dependencies > Add Local**), or from another package manifest:

```swift
.package(path: "../intl-ai/swift/IntlAi")
```

Use it from Swift:

```swift
import IntlAi

let intl = try IntlAi(configPath: "intl-ai.toml", workingDir: projectDir)
let report = try intl.check(options: CheckOptions(
    locales: [], keys: [], keysFile: nil,
    origin: nil, failOn: nil, noCache: false))
```

Or with an inline config instead of a file:

```swift
let intl = try IntlAi.fromConfigString(
    config: tomlString, format: .toml, workingDir: projectDir)
```

Every call is synchronous and blocking: `fill` performs provider I/O. Dispatch calls off the main thread (a `Task.detached`, a background `DispatchQueue`, or a dedicated actor). Errors surface as `IntlAiError` with a `kind` matching the CLI's error taxonomy.

Requirements:

- iOS 13.0+ or macOS 11.0+.
- Rust and Xcode to run `build-xcframework.sh` (not needed once the binary target ships as a release asset).

See the [package README](https://github.com/sigilco/intl-ai/tree/develop/swift/IntlAi) for the full API surface.

== tab "CLI build phase"

Store source locale files in a project directory, run `intl-ai fill` as a build phase, and copy the generated translations into your app bundle:

```
MyApp/
├── intl-ai.toml
├── locales/
│   ├── en.json
│   └── es.json
├── MyApp/
│   ├── MyApp.swift
│   └── Resources/
│       └── locales/
│           ├── en.json
│           └── es.json
└── MyApp.xcodeproj/
```

Add a build script phase:

1. Select your app target in Xcode.
2. Open **Build Phases** and add a new **Run Script** phase named **"Translate Locales"**.
3. Paste the following script:

```bash
set -e

# Run intl-ai fill to generate missing translations.
if command -v intl-ai &> /dev/null; then
  intl-ai fill --config "$SRCROOT/intl-ai.toml"
else
  echo "warning: intl-ai not found in PATH. Skipping translation."
fi

# Copy generated locale files into the app bundle.
LOCALES_DIR="$SRCROOT/locales"
DEST_DIR="$BUNDLE_RESOURCE_PATH/locales"

if [ -d "$LOCALES_DIR" ]; then
  mkdir -p "$DEST_DIR"
  cp -R "$LOCALES_DIR"/*.json "$DEST_DIR/"
fi
```

4. Drag the **Translate Locales** phase before **Copy Bundle Resources**.

Load translations at runtime with `Bundle.main.url(forResource:withExtension:)` or a `Bundle.main.decode(_:)` helper:

```swift
import Foundation

extension Bundle {
  func decode<T: Decodable>(_ file: String, as type: T.Type = T.self) -> T {
    guard let url = self.url(forResource: file, withExtension: nil) else {
      fatalError("Failed to locate \(file) in bundle.")
    }
    guard let data = try? Data(contentsOf: url) else {
      fatalError("Failed to load \(file) from bundle.")
    }
    let decoder = JSONDecoder()
    guard let loaded = try? decoder.decode(T.self, from: data) else {
      fatalError("Failed to decode \(file) from bundle.")
    }
    return loaded
  }
}

struct Localizations: Decodable {
  let hello: String
  let goodbye: String
}

let en = Bundle.main.decode("en.json", as: Localizations.self)
```

Requirements:

- `intl-ai` installed on your `PATH` (see [Installation](/guide/getting-started#installation)).
- `intl-ai.toml` at project root.

:::
