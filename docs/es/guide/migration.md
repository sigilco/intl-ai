---
title: Migración desde 0.4.x
description: Ruta de actualización desde los paquetes TypeScript 0.4.x al binario intl-ai de Rust y los shims @intl-ai/*.
---

# Migración desde 0.4.x

intl-ai 1.x reemplaza los paquetes de TypeScript con un único binario de Rust. Instálelo desde Homebrew, mise, el script de instalación o el wrapper npm `intl-ai`; no se requiere Node.js para ejecutarlo. Los paquetes `@intl-ai/*` son adaptadores delgados que localizan y ejecutan el binario dentro de su herramienta de compilación.

Esta página asigna cada concepto de 0.4.x a su equivalente en 1.x.

## Paquetes

| 0.4.x               | 1.x                                                                                           |
| ------------------- | --------------------------------------------------------------------------------------------- |
| `@intl-ai/cli`      | `intl-ai` binario (brew, mise, install.sh, o envoltorio npm)                                  |
| `@intl-ai/api`      | Eliminado; el binario es toda la superficie de la API                                         |
| `@intl-ai/unplugin` | `@intl-ai/unplugin` shim (ejecutar el binario en `buildStart`)                                |
| `@intl-ai/next`     | `@intl-ai/next` shim (`withIntlAi()` ejecuta el binario en la evaluación de la configuración) |

`@intl-ai/cli` y `@intl-ai/api` permanecen en npm, descontinuadas en sus últimas versiones 0.x. Si usaste `@intl-ai/api` programáticamente desde TypeScript, migra a ejecutar el binario (`intl-ai fill`, `intl-ai check`) o a configuraciones de proveedor `kind = "command"`.

## Instalar

```bash
brew install sigilco/intl-ai/intl-ai
# or
mise use -g intl-ai
# or the npm wrapper, which downloads the platform binary on install
npm install -D intl-ai
```

## Configuración

0.4.x leía `intl-ai.config.{ts,js,mjs}` o `.intl-ai.json` con claves camelCase. 1.x lee `intl-ai.toml`, `intl-ai.json`, `intl-ai.yaml`, o `intl-ai.yml` con claves snake_case.

### Mapeo de campos

| 0.4.x                               | 1.x                                                       |
| ----------------------------------- | --------------------------------------------------------- |
| `defaultLocale`                     | `source`                                                  |
| `locales` (menos el predeterminado) | `targets`                                                 |
| `localeDir`                         | `locale_dir`                                              |
| `glossary`                          | `glossary`                                                |
| `localeInstructions`                | `locale_instructions`                                     |
| `maxRetries`                        | `max_retries`                                             |
| `batchSize`                         | `batch_size`                                              |
| `processor`                         | `processor` (`passthrough` por defecto, `icu`)            |
| `format`                            | `format` (`json` predeterminado, `yaml`)                  |
| `provider`, `model`                 | `[provider]` `provider`, `model`                          |
| `apiKey`                            | `api_key` (interpolinar secretos, véase abajo)            |
| `baseURL`                           | `base_url`                                                |
| `modelParams`                       | `model_params`                                            |
| `kind = "agent"` + `agent`          | `[provider]` `kind = "command"` + `agent`                 |
| `quality.threshold`                 | `[[checks]] id = "judge"` `threshold` + `[quality]` bands |

### Ejemplo

0.4.x:

```json
{
  "defaultLocale": "en",
  "locales": ["en", "es", "fr"],
  "localeDir": "locales",
  "provider": "openai",
  "model": "gpt-4o",
  "apiKey": "sk-..."
}
```

1.x:

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
model = "gpt-4o"
api_key = "${env:OPENAI_API_KEY}"
```

::: warning
No pegar las claves API en `intl-ai.toml`. Usa la interpolación `${env:VAR}` o `${file:PATH}`; `config validate` imprime los secretos enmascarados.
:::

## Archivo de bloqueo

0.4.x escribió la procedencia en un único `intl-ai.lock.json`. 1.x escribe archivos `intl-ai.lock.d/<locale>.toml` fragmentados. No hay un comando `intl-ai migrate`: las migraciones se envían como documentos que ejecutas manualmente o entregas a un agente. En la primera ejecución, las claves existentes en tus archivos de locale se registran como propiedad humana: nunca se sobrescriben ni se re-traducen. Solo las claves faltantes reciben rellenos de IA. Eso significa que la migración práctica es:

1. Instala el binario y traduce tu configuración como se indicó anteriormente.
2. Ejecuta `intl-ai fill` una vez; escribe nuevos fragmentos de `intl-ai.lock.d/` y completa solo lo que falta.
3. Elimina `intl-ai.lock.json`.

Si anteriormente confiabas en el seguimiento de origen de IA para volver a traducir las fuentes cambiadas (semántica `--stale`), esas claves son de propiedad humana después de la migración; utilice `intl-ai fill --regenerate` o `--include-human` de forma selectiva para volver a traducirlas una vez.

## Comandos

| 0.4.x                  | 1.x                                                       |
| ---------------------- | --------------------------------------------------------- |
| `intl-ai fill`         | `intl-ai fill` (`--dry-run` previsualizaciones)           |
| `intl-ai fill --force` | `intl-ai fill --regenerate` / `--include-human`           |
| `intl-ai check`        | `intl-ai check` (`--fail-on` para controles de severidad) |
| —                      | `intl-ai init`, `status`, `config validate`               |

## Validación y calidad

La versión 0.4.x `quality.threshold` ahora es un sistema de comprobación de primera clase: `[[checks]]` entradas para `icu`, `placeholder-parity`, `dialect:<locale>`, y `judge` (con `threshold`/`weight` por comprobación), agrupado por `[quality]` `fail_below`/`review_below` rangos en pasar/revisar/fallar por clave. Consulte [Configuration](/es/guide/configuration/) y [Observability](/es/guide/observability/).

## Adaptadores

Si integraste intl-ai en un empaquetador, los paquetes shim cambiaron de forma:

- `@intl-ai/unplugin`: mismo nombre, pero las opciones ahora describen el comportamiento del binario (`fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `strict`, `config`, `cwd`, `bin`, `args`).
- `@intl-ai/next`: `withIntlAi(nextConfig, options)` ejecuta el binario durante la evaluación de configuración, por lo que funciona tanto en webpack como en Turbopack.
- `@intl-ai/expo`: plugin de configuración que ejecuta el binario durante `prebuild`.

Todos los adaptadores declaran `intl-ai` como una dependencia para que el binario se resuelva dentro de proyectos instalados con npm.
