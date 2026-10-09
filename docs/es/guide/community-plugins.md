---
title: Integraciones comunitarias
description: Los shims soportados por Tier-1 frente a los plugins respaldados por la comunidad, y cómo construir el tuyo propio.
---

# Integraciones comunitarias

El contrato de intl-ai es pequeño: una pila ejecuta `intl-ai fill` en el punto del ciclo de vida donde deben existir las traducciones, y opcionalmente `intl-ai check --fail-on` como puerta. Todo lo demás (qué herramienta de construcción, qué biblioteca i18n, qué idioma) es intercambiable. Eso hace que las integraciones mantenidas por la comunidad sean una combinación natural.

## Niveles de soporte

**Tier 1, mantenido en este repositorio** y versionado al paso con el binario:

| Integración         | Pila                                        | Forma                                  |
| ------------------- | -------------------------------------------- | -------------------------------------- |
| `intl-ai`           | cualquiera                                          | el propio binario                      |
| `@intl-ai/unplugin` | Vite, Webpack, Rollup, esbuild, Rspack, Farm | adaptador unplugin, se ejecuta en `buildStart` |
| `@intl-ai/next`     | Next.js (webpack y Turbopack)              | `withIntlAi()` wrapper de `next.config`   |
| `@intl-ai/expo`     | Expo / React Native                          | plugin de configuración, se ejecuta durante prebuild    |

Los shims de Tier-1 reciben correcciones publicadas y la misma superficie de opciones (`fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `strict`, `config`, `cwd`, `bin`, `args`).

**Soportado por la comunidad**: todo lo demás, mantenido por su autor. Un paquete `intl_ai` para Flutter en pub.dev, una tarea de Gradle, un script de fase de compilación de Xcode, un plugin de WordPress o Rails, un envoltorio Metro: todos son bienvenidos. Revisamos y enlazamos aquí y en el README, pero son propiedad de sus autores para correcciones y lanzamientos.

## Construir un shim

Un buen shim hace tres cosas:

1. **Locate the binary.** Para paquetes de npm, depende del paquete `intl-ai` (instala el binario de plataforma en postinstall) y resuélvelo como [`@intl-ai/unplugin` lo hace](https://github.com/sigilco/intl-ai/blob/develop/packages/unplugin/src/run.ts): opción `bin`, entorno `INTL_AI_BIN`, el paquete de npm, luego `PATH`. Fuera de npm, documenta los canales de instalación (script de instalación, brew, mise) y resuelve el binario de la misma manera.
2. **Generar en el ciclo de vida correcto.** `intl-ai fill` donde deberían existir traducciones (inicio de compilación, prebuild, pipeline de activos), más `intl-ai check --fail-on <kinds>` si el shim actúa como puerta de compilación.
3. **Exponer los controles de calidad.** Mapea la superficie de calidad de la CLI a las convenciones de tu plataforma: la puerta `[fill].validate`, umbral `judge` mediante `--judge-threshold`, severidades `--fail-on`, y un opt-out de desarrollo. No reimplementes la validación en el shim; siempre delega en el binario.

¿Prefieres traducir formatos distintos de JSON/YAML (ARB, `.strings`, `.xml`, PO)? Registra un plugin de formato de ejecución mediante `[[formats]]` en `intl-ai.toml`, sin necesidad de Rust (ver [Format plugins](/es/guide/format-plugins/)). Para enviar un plugin reutilizable para la comunidad de un formato, abre un issue o PR para enlazarlo aquí.

## Ser incluido

Abre un PR que agregue tu integración a esta página y a la sección de la comunidad del README, con una descripción de una línea, comando de instalación y enlace al repositorio. Requisitos: código abierto, mantenido (commits recientes), y el shim delega al binario `intl-ai` en lugar de reimplementar fill o chequeos.
