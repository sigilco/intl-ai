---
title: bibliotecas i18n
description: intl-ai genera archivos de locale en tiempo de construcción. Úsalo con vue-i18n, i18next o cualquier biblioteca i18n.
---

# bibliotecas i18n

intl-ai genera archivos de traducción en tiempo de compilación. Usted elige la biblioteca de i18n en tiempo de ejecución. Esta página enumera las opciones compatibles y su compatibilidad con ICU.

## ICU MessageFormat

La mayoría de las bibliotecas i18n modernas hablan ICU MessageFormat. Establece `processor = "icu"` en `intl-ai.toml` para que el modelo genere sintaxis ICU; es el contrato que valida la comprobación `icu`.

La principal excepción es i18next, que utiliza interpolación con `{{var}}` y sufijos plurales en lugar de sintaxis ICU.

## Compatibilidad de la biblioteca

| Librería                       | ICU nativo                | Configuración                    | Mejor para                               |
| ----------------------------- | ------------------------- | ------------------------ | -------------------------------------- |
| react-intl                    | Sí                       | Ninguno                     | Aplicaciones de React que necesitan ICU               |
| @formatjs/intl                | Sí                       | Ninguno                     | Tiempo de ejecución ICU agnóstico al framework         |
| lingui                        | Sí                       | Macro de compilación              | Seguridad en tiempo de compilación, cualquier framework     |
| i18next-icu                   | Sí (a través de plugin)          | Instala `i18next-icu`    | Aplicaciones existentes de i18next que migran a ICU |
| vue-i18n + intl-messageformat | Sí (vía compilador personalizado) | Personalizado `messageCompiler` | Apps de Vue que necesitan ICU                 |
| svelte-i18n                   | Parcial                   | Llamadas manuales al formateador   | Apps de Svelte                            |
| @cookbook/solid-intl          | Sí                       | Ninguno                     | Aplicaciones sólidas que necesitan ICU               |
| @lit/localize                 | Parcial                   | Solución alternativa de la comunidad     | Componentes web                         |
| typesafe-i18n                 | No                        | N/A                      | i18n seguro de tipos sin ICU             |
| rosetta                       | No                        | N/A                      | Huella pequeña (298 bytes)             |
| @solid-primitives/i18n        | No                        | N/A                      | i18n mínimo de Solid                     |

## Configuración

Set `processor = "icu"` in `intl-ai.toml` cuando tu biblioteca de destino use ICU MessageFormat. La mayoría de las bibliotecas modernas lo hacen.

Si usas i18next, omite la configuración del procesador (passthrough preserva su estilo `{{var}}`).

```toml
# intl-ai.toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]
processor = "icu"   # recommended for most libraries
```

## Guías por librería

- [Vue (vue-i18n)](/es/guide/i18n-libraries/vue-i18n)
- [i18next](/es/guide/i18n-libraries/i18next)

## Elegir una biblioteca

| Necesidad                     | Usar                                           |
| ------------------------ | --------------------------------------------- |
| React + ICU              | `react-intl`                                  |
| Vue + ICU                | `vue-i18n` con el compilador `intl-messageformat` |
| sintaxis i18next (sin ICU)  | `i18next` + `react-i18next`                   |
| Seguridad de tipos en tiempo de compilación | `lingui` o `typesafe-i18n`                   |
| El bundle más pequeño          | `rosetta`                                     |
