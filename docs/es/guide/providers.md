---
title: Proveedores
description: Referencia del proveedor intl-ai. HTTP (compatible con OpenAI), comando (agentes CLI locales) y transportes de replay.
---

# Proveedores

intl-ai traduce las claves de idioma llamando a un modelo de IA. El bloque `[provider]` en `intl-ai.toml` selecciona el transporte; todo lo demás (contrato de prompt, agrupamiento, reintentos, el juez) se comparte.

## `kind = "http"`

Un punto final de completaciones de chat compatible con OpenAI. La superficie más amplia: OpenAI, OpenRouter, LM Studio, Ollama, vLLM, Azure OpenAI y la mayoría de los proveedores alojados exponen esta forma.

```toml
[provider]
kind = "http"
provider = "openai"                 # wire-shape preset (default "openai")
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"

[provider.model_params]
temperature = 0.2                   # extra request-body params, spread last
```

| Campo          | Descripción                                                                                    |
| -------------- | ---------------------------------------------------------------------------------------------- |
| `provider`     | Nombre de la plantilla de cable; por defecto `openai`.                                                  |
| `model`        | Identificador del modelo enviado en el cuerpo de la solicitud.                                                     |
| `api_key`      | Enviado como `Authorization: Bearer <key>`. Soporta la interpolación de `${env:VAR}` y `${file:PATH}`. |
| `base_url`     | Anulación del punto de conexión; por defecto, se utiliza la URL canónica del proveedor.                                   |
| `model_params` | Parámetros adicionales del cuerpo de la solicitud, colocados al final para que tengan prioridad.                                            |

Los reintentos de las solicitudes se hacen con retroceso hasta `max_retries` (valor predeterminado 3).

## `kind = "command"`

Ejecuta un agente CLI local y lee las traducciones desde su salida estándar. Úsalo para proveedores con una forma de cable diferente, CLIs de agentes que ya tienes (un `agent` preset nombra uno conocido), o herramientas personalizadas.

```toml
[provider]
kind = "command"
command = "my-translator"
args = ["--json"]
prompt_via = "stdin"        # or "argv" to append the prompt as the last arg
cwd = "."
timeout_ms = 300000
```

| Campo              | Descripción                                                      |
| ------------------ | ---------------------------------------------------------------- |
| `command`          | Línea de comandos de formato libre. Solo configuración raíz, nunca mediante `extends`.   |
| `agent`            | Nombre de la plantilla desde la tabla de agentes incorporados (kebab-case).          |
| `args`             | Argumentos adicionales pasados al comando.                           |
| `prompt_via`       | `stdin` (por defecto) o `argv`.                                     |
| `cwd`              | Directorio de trabajo, resuelto contra el directorio del archivo de configuración. |
| `timeout_ms`       | Presupuesto de tiempo real antes de SIGTERM, luego SIGKILL (por defecto 300000). |
| `max_stdout_bytes` | Límite de búfer de stdout                                             |

Dado que `command` puede ejecutar programas arbitrarios, se rechaza en archivos de configuración con `extends`: los datos de una dependencia no deben ejecutar código.

## `kind = "replay"`

Un cassette JSON de traducciones predefinidas, para pruebas y demos. Sin red, sin credenciales, salida determinística.

```toml
[provider]
kind = "replay"
file = "cassette.json"
```

El cassette asigna `{ "<locale>": { "<source value>": "<translation>" } }` y se resuelve respecto al directorio del archivo de configuración.

## El contrato del prompt

Todo transporte responde al mismo prompt: las claves aplanadas del locale de origen más instrucciones, términos del glosario y el contrato de marcador de posición (`processor = "icu"` exige ICU MessageFormat). La respuesta debe ser un objeto JSON de traducciones. `fill` lo analiza estrictamente; la comprobación `judge` puntúa la calidad a través del mismo transporte.

Consulta [Configuration](/es/guide/configuration/) para el esquema completo.
