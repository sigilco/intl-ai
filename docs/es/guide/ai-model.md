---
title: Configuración del modelo de IA
description: Configura cualquier modelo de IA para intl-ai. Cualquier punto final compatible con OpenAI, un agente CLI local o una cassette de reproducción.
---

# Configuración del modelo de IA

La traducción es estructurada y sigue instrucciones. Los modelos de bajo coste la manejan bien, así que no necesitas un modelo de primera línea.

## Elige un proveedor

Tres caminos funcionan: un servidor local, un punto final en la nube o un aggregador. Los tres usan `kind = "http"` contra una API de completaciones de chat compatible con OpenAI.

### Local: LM Studio u Ollama

[LM Studio](https://lmstudio.ai) y [Ollama](https://ollama.com) ejecutan modelos localmente y exponen un servidor compatible con OpenAI. Esto es ideal para desarrollo, pruebas y trabajo sensible a la privacidad.

```toml
[provider]
kind = "http"
base_url = "http://127.0.0.1:1234/v1"
model = "your-local-model"
api_key = "lm-studio"
```

### Nube: Proveedores compatibles con OpenAI

Cualquier proveedor con un punto final compatible con OpenAI funciona de la misma manera: configura `base_url`, `model` y `api_key`.

```toml
[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

Anthropic, Google, Azure OpenAI, Cohere, Mistral y otros exponen superficies compatibles con OpenAI (de primera parte o a través de un proxy). Apunta `base_url` al tuyo y configura `model` en consecuencia. Para una API con una forma de cable diferente, usa el proveedor `command` para ejecutar un agente local en su lugar; consulte [Proveedores](/es/guide/providers/).

### Agregador: OpenRouter

[OpenRouter](https://openrouter.ai) te da una clave API para muchos proveedores y un tier gratuito.
Un modelo gratuito estable en el momento de la escritura es `google/gemini-2.0-flash-exp:free`.
Si deja de funcionar, consulta la lista de modelos gratuitos de OpenRouter y actualiza esta única referencia.

```toml
[provider]
kind = "http"
base_url = "https://openrouter.ai/api/v1"
model = "google/gemini-2.0-flash-exp:free"
api_key = "${env:OPENROUTER_API_KEY}"
```

## Ajuste de la solicitud

`model_params` agrega campos adicionales al cuerpo de la solicitud al final, de modo que tus parámetros ganan sobre los valores predeterminados:

```toml
[provider.model_params]
temperature = 0.2
max_tokens = 1024
```

## Ventana de contexto

Usa un modelo con una ventana de contexto mínima de 16 000 tokens. Cada proveedor listado anteriormente supera esto.

## Solución de problemas

Si la traducción falla, comprueba:

- La interpolación de `api_key` se resuelve (`intl-ai config validate` lo muestra enmascarado).
- El servidor del proveedor es accesible desde tu máquina (`base_url`).
- Tu identificador `model` coincide con la documentación del proveedor.
