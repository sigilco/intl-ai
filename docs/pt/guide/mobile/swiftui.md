---
title: SwiftUI
description: Tradução de IA para i18n de SwiftUI através do pacote SwiftPM IntlAi ou uma fase de compilação do Xcode. As traduções ocorrem em tempo de compilação.
---

# SwiftUI

Pode integrar `intl-ai` num projeto SwiftUI de duas formas. O pacote SwiftPM `IntlAi` incorpora o motor para que o código do aplicativo ou ferramentas possa chamar `fill`, `check` e `status` em processo, enquanto a CLI é executada como uma fase de compilação do Xcode para tradução pura em tempo de compilação sem API em tempo de execução.

::: tabs

== tab "SwiftPM package"

O pacote `IntlAi` encontra-se em `swift/IntlAi` no repositório intl-ai e envolve o motor Rust num XCFramework estático para iOS e macOS. Até que o ativo de lançamento v0.7.0 seja publicado, compile o XCFramework uma vez e dependa do pacote por caminho local:

```sh
git clone https://github.com/sigilco/intl-ai
intl-ai/swift/IntlAi/scripts/build-xcframework.sh
```

Em seguida, adicione `swift/IntlAi` como um pacote local no Xcode (**Ficheiro > Adicionar Dependências de Pacote > Adicionar Local**), ou a partir de outro manifesto de pacote:

```swift
.package(path: "../intl-ai/swift/IntlAi")
```

Utilize-o a partir de Swift:

```swift
import IntlAi

let intl = try IntlAi(configPath: "intl-ai.toml", workingDir: projectDir)
let report = try intl.check(options: CheckOptions(
    locales: [], keys: [], keysFile: nil,
    origin: nil, failOn: nil, noCache: false))
```

Ou com uma configuração embutida em vez de um ficheiro:

```swift
let intl = try IntlAi.fromConfigString(
    config: tomlString, format: .toml, workingDir: projectDir)
```

Toda chamada é síncrona e bloqueante: `fill` realiza I/O do fornecedor. Despache chamadas fora da thread principal (um `Task.detached`, uma `DispatchQueue` em segundo plano, ou um ator dedicado). Erros aparecem como `IntlAiError` com um `kind` que corresponde à taxonomia de erros da CLI.

Requisitos:

- iOS 13.0+ ou macOS 11.0+.
- Rust e Xcode para executar `build-xcframework.sh` (não necessário uma vez que o alvo binário é enviado como um ativo de lançamento).

Consulte o [README do pacote](https://github.com/sigilco/intl-ai/tree/develop/swift/IntlAi) para a superfície completa da API.

== tab "CLI build phase"

Armazene os ficheiros de localidade de origem num diretório do projeto, execute `intl-ai fill` como uma fase de compilação e copie as traduções geradas para o pacote do seu aplicativo:

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

Adicione uma fase de script de compilação:

1. Selecione o alvo do seu aplicativo no Xcode.
2. Abra **Fases de Compilação** e adicione uma nova fase **Executar Script** denominada **"Traduzir Localidades"**.
3. Cole o seguinte script:

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

4. Arraste a fase **Traduzir Localidades** antes de **Copiar Recursos do Pacote**.

Carregue traduções em tempo de execução com `Bundle.main.url(forResource:withExtension:)` ou um auxiliar `Bundle.main.decode(_:)` :

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

- `intl-ai` instalado no seu `PATH` (veja [Installation](/pt/guide/getting-started#installation)).
- `intl-ai.toml` na raiz do projeto.

:::
