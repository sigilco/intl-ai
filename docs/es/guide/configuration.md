---
title: Configuración
description: Referencia del archivo de configuración intl-ai. intl-ai.toml, .json, o .yaml, validado contra un esquema publicado.
---

# Configuración

intl-ai lee un único archivo de configuración, `intl-ai.toml`, `intl-ai.json`, `intl-ai.yaml`, o `intl-ai.yml`, descubierto en la raíz de tu proyecto (la primera coincidencia gana). Pasa `--config <path>` para elegir uno explícitamente, o `--config -` para leer TOML desde stdin.

El contrato se valida frente a un esquema JSON publicado disponible en `https://intl-ai.illo.fyi/schema/v1.json` (`intl-ai config schema` imprime el mismo esquema).

## Configuración mínima

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

## Campos requeridos

### `locale_dir`

Directorio que contiene los archivos de locale, resuelto contra el directorio del archivo de configuración. Un archivo por locale: `${locale_dir}/${locale}.json` (o `.yaml`).

### `source`

Código de locale de origen, por ejemplo, `"en"`.

### `targets`

Códigos de locale objetivo a completar y verificar.

### `[provider]`

Cómo se producen las traducciones. Tres tipos:

- `kind = "http"`: un punto de extremo de completaciones de chat compatible con OpenAI (`provider`/`model`/`api_key`/`base_url`/`model_params`). Consulte [Providers](/es/guide/providers/).
- `kind = "command"`: un agente CLI local (`command`/`args`/`agent`/`prompt_via`/`cwd`/`timeout_ms`). Solo permitido en el archivo de configuración raíz, nunca mediante `extends`.
- `kind = "replay"`: un archivo JSON de cassette de traducciones predefinidas, para pruebas y demos.

## Campos opcionales

### `api_key` interpolación

Los secretos se interpolan al cargar: `${env:VAR}` lee una variable de entorno, `${file:PATH}` lee un archivo. `config validate` los imprime enmascarados.

```toml
api_key = "${env:OPENAI_API_KEY}"
```

### `glossary`

Pares fijos de término-traducción inyectados en cada solicitud de traducción.

```toml
[glossary]
React = "React"
TypeScript = "TypeScript"
```

### `locale_instructions`

Instrucciones de estilo o dialecto libres por localización, enviadas al modelo y a la comprobación del juez. Orden de resolución: localización exacta (`en-GB`), luego subtipo de idioma (`en`), luego `*` como opción general.

```toml
[locale_instructions]
en-GB = "Use British spelling (colour, organise)."
"*" = "Keep a formal tone."
```

Cambiar una instrucción no retraduce las entradas existentes. Retraduzca una configuración regional con `intl-ai fill --locale en-GB --regenerate`.

### `batch_size`

Máximo de entradas de origen por solicitud de traducción. El valor predeterminado es ilimitado (todas las claves en un lote). Reduzca para modelos con ventanas de contexto más pequeños o fallas más granulares por clave.

### `max_retries`

Intentos de transporte por solicitud antes de que falle el lote. Predeterminado `3`, máximo `10`.

### `processor`

Pista de contrato de marcador de posición enviada al modelo. `icu` selecciona ICU MessageFormat; el predeterminado es passthrough.

### `format`

El formato del archivo local para nuevos archivos es: `json` (predeterminado), `yaml`, o un nombre de `[[formats]]`. La extensión propia de un archivo existente siempre prevalece, por lo que los directorios mixtos funcionan. Los nombres desconocidos no superan la validación.

### `extends`

Configurar configuraciones: una configuración raíz puede extender una base (lista o ruta única). Los archivos extendidos no pueden definir `provider.command`, comprobaciones de `exec` o formatos de `exec`: los archivos de datos de una dependencia no deben iniciar programas.

## Formatos personalizados

### `[[formats]]`

Formatos de locale externos registrados junto con los integrados. Consulte [Format plugins](/es/guide/format-plugins/) para el protocolo y una implementación de referencia.

```toml
[[formats]]
name = "xml"
exec = "python3"
args = ["tools/strings_xml.py"]
extension = "xml"
extensions = ["axml"]     # optional extra claimed extensions
cwd = "tools"            # optional; defaults to the config directory
timeout_ms = 60000       # optional
max_stdout_bytes = 10485760
```

## Comprobaciones y puertas de calidad

### `[[checks]]`

Comprobaciones de validación ejecutadas por `intl-ai check` y el fill gate.

```toml
[[checks]]
id = "icu"

[[checks]]
id = "placeholder-parity"

[[checks]]
id = "judge"
threshold = 0.85
weight = 2.0
```

IDs integrados: `icu`, `placeholder-parity`, `dialect:<locale>`, `judge`. Una entrada de comprobación también puede apuntar a una especificación YAML declarativa (`spec = "checks/brand.yaml"`) o a un comando externo (`exec = "./my-check"` con `args`/`cwd`/`timeout_ms`/`max_stdout_bytes`, solo config raíz). `threshold` es solo para el juez (puntuación por debajo de él se convierte en un hallazgo `invalid`, por defecto `0.8`); `weight` alimenta el agregado `[quality]`.

### `[check]`

```toml
[check]
fail_on = ["missing", "stale", "invalid"]
cache = true
```

`fail_on` lista los tipos de hallazgos que fallan `intl-ai check`. `cache` alterna la caché incremental de hallazgos (`.intl-ai/check-cache.json`).

### `[fill]`

```toml
[fill]
validate = ["icu", "placeholder-parity", "judge"]
```

La puerta de entrada shift-left: estas comprobaciones se ejecutan dentro de `fill` entre la traducción y la adopción. Solo las comprobaciones que permiten comentarios son válidas (`icu`, `placeholder-parity`, `judge`). Las claves que no superan la comprobación obtienen una ronda de corrección; las claves no resueltas se adoptan de todos modos, se registran en `quality.unresolved` en el archivo de bloqueo y `check` sigue marcándolas. Anula por ejecución con `--validate`/`--no-validate`.

### `[quality]`

```toml
[quality]
fail_below = 0.5
review_below = 0.8
```

Puntuación ponderada por clave sobre las comprobaciones configuradas (comprobaciones binarias puntuación 1.0/0.0, el juez puntuación real 0..1). Por debajo de `fail_below` es un hallazgo `invalid`; por debajo de `review_below` marca la clave como no revisada.

## IntelliSense del editor

Añade `"$schema": "https://intl-ai.illo.fyi/schema/v1.json"` a una configuración JSON para autocompletado y validación.

## Validación de CI

```bash
intl-ai config validate   # resolves extends, masks secrets, fails on bad shape
```

Consulte [Providers](/es/guide/providers/) para los tipos de proveedor y [AI model setup](/es/guide/ai-model/) para la selección de endpoints.
