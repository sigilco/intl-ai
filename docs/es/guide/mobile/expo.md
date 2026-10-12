---
title: Expo
description: Traducción IA en tiempo de compilación para Expo i18n. Las traducciones ocurren durante la precompilación, sin tiempo de ejecución.
---

# Expo

`@intl-ai/expo` es un plugin de configuración que ejecuta el binario `intl-ai` mientras Expo evalúa los plugins de configuración durante `expo prebuild`, `eas build` y `expo run:*`. Las traducciones se escriben en disco antes de que Metro empaquete tu aplicación, por lo que no hay ninguna sobrecarga en tiempo de ejecución.

## Instalar

```bash
pnpm add -D @intl-ai/expo
```

El paquete npm `intl-ai` va como dependencia y descarga el binario de la plataforma al instalar. Para usar un binario que hayas instalado de otra forma (brew, mise, script de instalación), configura `INTL_AI_BIN` o pasa `bin`.

## Configurar `app.json`

```json
{
  "expo": {
    "plugins": ["@intl-ai/expo"]
  }
}
```

Con opciones:

```json
{
  "expo": {
    "plugins": [
      [
        "@intl-ai/expo",
        {
          "failOn": ["missing", "invalid"],
          "judgeThreshold": 0.9
        }
      ]
    ]
  }
}
```

## Crea `intl-ai.toml`

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${OPENAI_API_KEY}"
```

Ver [Configuración](/es/guide/configuration/) para el esquema completo.

## Ejecutar preconstrucción

```bash
expo prebuild
```

El plugin se registra en las plataformas iOS y Android y ejecuta el pipeline una vez, con la raíz de la aplicación como directorio de trabajo.

## Opciones

| Opción           | Tipo                  | Predeterminado        | Descripción                                                  |
| ---------------- | --------------------- | --------------------- | ------------------------------------------------------------ |
| `fill`           | `boolean`             | `true`                | Ejecutar `intl-ai fill`                                      |
| `failOn`         | `string \| string[]`  | —                     | Ejecuta `intl-ai check --fail-on <kinds>` después de fill    |
| `validate`       | `boolean \| string[]` | `true`                | Puerta de validación en tiempo de llenado                    |
| `judgeThreshold` | `number`              | —                     | Umbral de puntuación del juez dentro de la puerta de relleno |
| `dev`            | `boolean`             | `true`                | `false` omite cuando `NODE_ENV !== "production"`             |
| `strict`         | `boolean`             | `true`                | `false` advierte en lugar de fallar la compilación previa    |
| `config`         | `string`              | —                     | Ruta a `intl-ai.toml`/`.json`/`.yaml`                        |
| `cwd`            | `string`              | raíz de la aplicación | Directorio de trabajo para el binario                        |
| `bin`            | `string`              | —                     | Ruta explícita al binario de `intl-ai`                       |
| `args`           | `string[]`            | —                     | Argumentos extra agregados a `intl-ai fill`                  |

Orden de resolución binaria: opción `bin` → variable de entorno `INTL_AI_BIN` → el paquete npm `intl-ai` → `PATH`.

## Uso en tiempo de ejecución

Carga los archivos JSON generados directamente con tu biblioteca i18n preferida (`i18next`, `react-intl`, etc.). El plugin solo escribe traducciones; no impone una API en tiempo de ejecución.

## Ejemplo

Ver [`examples/expo`](https://github.com/sigilco/intl-ai/tree/main/examples/expo) para una aplicación completa y funcional.
