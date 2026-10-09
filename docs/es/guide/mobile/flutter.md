---
title: Flutter
description: Traducción con IA para Flutter i18n en tiempo de compilación. Las traducciones ocurren durante la compilación, con tiempo de ejecución cero.
---

# Flutter

La integración de Flutter se proporciona como un constructor autoconteniendo [`build_runner`](https://pub.dev/packages/build_runner) en [`examples/flutter/plugin`](https://github.com/sigilco/intl-ai/tree/main/examples/flutter/plugin). Ejecuta `intl-ai fill` durante `flutter pub run build_runner build` y tiene una sobrecarga de tiempo de ejecución cero porque todas las traducciones se escriben en disco antes de que la aplicación Flutter sea empaquetada.

## Copiar el builder

Copie `examples/flutter/plugin/` en su proyecto Flutter (por ejemplo, en `./tools/intl_ai_builder/`). El builder no se publica como un paquete, por lo que usted lo posee y puede personalizar el código.

## Configurar `build.yaml`

Agregue el builder al `build.yaml` de su app:

```yaml
targets:
  $default:
    builders:
      intl_ai_flutter|intl_ai:
        enabled: true
        options:
          configPath: "intl-ai.toml"
          executable: "intl-ai"
          verbose: false
```

Actualiza la ruta de `import` en `build.yaml` para que coincida con donde copiaste el constructor, por ejemplo:

```yaml
builders:
  intl_ai:
    import: "package:my_app/tools/intl_ai_builder/lib/builder.dart"
```

## Crear `intl-ai.toml`

```toml
locale_dir = "assets/locales"
source = "en"
targets = ["es"]

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

## Ejecutar build_runner

```bash
flutter pub get
flutter pub run build_runner build --delete-conflicting-outputs
```

El builder invoca:

```bash
intl-ai fill --config intl-ai.toml
```

## Opciones del constructor

| Opción       | Tipo     | Predeterminado        | Descripción                                                                                              |
| ------------ | -------- | -------------- | -------------------------------------------------------------------------------------------------------- |
| `configPath` | `String` | `intl-ai.toml` | Ruta al archivo de configuración, relativa a la raíz del paquete.                                                  |
| `executable` | `String` | `intl-ai`      | Comando utilizado para invocar la CLI. Use `npx intl-ai` o `bunx intl-ai` si no desea una instalación global. |
| `verbose`    | `bool`   | `false`        | Reenviar la salida de la CLI al registro del ejecutor de compilación.                                                              |

## Cómo funciona

El constructor se registra en el activo `$package$` y escribe un archivo marcador `intl-ai.done`. Cuando `build_runner` se ejecuta, invoca la CLI de `intl-ai` una vez por paquete. Agrega `intl-ai.done` a tu `.gitignore`; solo se utiliza para realizar el seguimiento de la finalización de la compilación.

## Uso en tiempo de ejecución

Carga los archivos JSON generados con tu biblioteca i18n preferida (`easy_localization`, `i18n_extension`, `flutter_gen`, etc.). El constructor solo escribe traducciones; no impone una API de tiempo de ejecución.

## Requisitos

- Dart SDK 3.0+
- `intl-ai` CLI en `PATH`
- Un archivo `intl-ai.toml` y un directorio de locale

## Ejemplo

Véase [`examples/flutter`](https://github.com/sigilco/intl-ai/tree/main/examples/flutter) para una aplicación completa y funcional.
