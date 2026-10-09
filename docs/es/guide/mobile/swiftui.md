---
title: SwiftUI
description: Traducción de IA para i18n de SwiftUI a través del paquete SwiftPM IntlAi o de una fase de compilación de Xcode. Las traducciones ocurren en tiempo de compilación.
---

# SwiftUI

Puedes integrar `intl-ai` en un proyecto SwiftUI de dos formas. El paquete SwiftPM `IntlAi` incrusta el motor para que el código de la aplicación o de herramientas pueda llamar a `fill`, `check` y `status` en el proceso, mientras que la CLI se ejecuta como una fase de compilación de Xcode para una traducción pura en tiempo de compilación sin API en tiempo de ejecución.

::: tabs

== tab "SwiftPM package"

El paquete `IntlAi` se encuentra en `swift/IntlAi` en el repositorio intl-ai y envuelve el motor Rust en un XCFramework estático para iOS y macOS. Hasta que se publique el activo de lanzamiento v0.7.0, construya el XCFramework una vez y dependa del paquete por la ruta local:

```sh
git clone https://github.com/sigilco/intl-ai
intl-ai/swift/IntlAi/scripts/build-xcframework.sh
```

Añade `swift/IntlAi` como un paquete local en Xcode (**Archivo > Agregar dependencias de paquetes > Agregar local**), o desde otro manifiesto de paquete:

```swift
.package(path: "../intl-ai/swift/IntlAi")
```

Úsalo desde Swift:

```swift
import IntlAi

let intl = try IntlAi(configPath: "intl-ai.toml", workingDir: projectDir)
let report = try intl.check(options: CheckOptions(
    locales: [], keys: [], keysFile: nil,
    origin: nil, failOn: nil, noCache: false))
```

O con una configuración en línea en lugar de un archivo:

```swift
let intl = try IntlAi.fromConfigString(
    config: tomlString, format: .toml, workingDir: projectDir)
```

Cada llamada es síncrona y bloqueante: `fill` realiza E/S del proveedor. Envía las llamadas fuera del hilo principal (un `Task.detached`, una `DispatchQueue` en segundo plano o un actor dedicado). Los errores aparecen como `IntlAiError` con un `kind` que coincide con la taxonomía de errores de la CLI.

Requisitos:

- iOS 13.0+ o macOS 11.0+.
- Rust y Xcode para ejecutar `build-xcframework.sh` (no es necesario una vez que el objetivo binario se distribuya como un activo de lanzamiento).

Consulte el [README del paquete](https://github.com/sigilco/intl-ai/tree/develop/swift/IntlAi) para ver la superficie API completa.

== tab "CLI build phase"

Almacena los archivos de locale fuente en un directorio de proyecto, ejecuta `intl-ai fill` como una fase de compilación y copia las traducciones generadas en tu bundle de la aplicación:

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

Añadir una fase de script de compilación:

1. Seleciona tu objetivo de aplicación en Xcode.
2. Abre **Fases de compilación** y añade una nueva fase **Ejecutar script** con el nombre **"Traducir locales"**.
3. Pega el siguiente script:

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

4. Arrastra la fase **Traducir locales** antes de **Copiar recursos del bundle**.

Carga las traducciones en tiempo de ejecución con `Bundle.main.url(forResource:withExtension:)` o un helper `Bundle.main.decode(_:)`:

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

Requisitos:

- `intl-ai` instalado en tu `PATH` (ver [Installation](/es/guide/getting-started#installation)).
- `intl-ai.toml` en la raíz del proyecto.

:::
