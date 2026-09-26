---
title: SwiftUI
description: Build-time AI translation for SwiftUI i18n via Xcode build phases. Zero runtime overhead.
---

# SwiftUI

You can integrate `intl-ai` into a SwiftUI project by running the CLI as an Xcode build script phase. All translations happen at build time, so there is zero runtime overhead.

## Project layout

```
MyApp/
├── intl-ai.config.json
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

Store source locale files in a project directory, then copy the generated translations into your app bundle as a build step.

## Add a build script phase

1. Select your app target in Xcode.
2. Open **Build Phases** and add a new **Run Script** phase named **"Translate Locales"**.
3. Paste the following script:

```bash
set -e

# Run intl-ai fill to generate missing translations.
if command -v intl-ai &> /dev/null; then
  intl-ai fill --config "$SRCROOT/intl-ai.config.json"
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

## Load translations at runtime

Use `Bundle.main.url(forResource:withExtension:)` or `Bundle.main.decode(_:)` helpers to load JSON files from the bundle:

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

## Requirements

- `intl-ai` installed on your `PATH` (see [Installation](/guide/getting-started#installation)).
- `intl-ai.config.json` at project root.

## Example

See the project layout above and adapt it to your own SwiftUI app. `intl-ai` only writes translations; it does not impose a runtime API.
