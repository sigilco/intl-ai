---
title: Observabilidad
description: Supervise el pipeline de intl-ai. Informes JSON, tipos de hallazgos, puntuaciones de calidad y comprobaciones exec como ganchos personalizados.
---

# Observabilidad

El binario `intl-ai` emite salida legible por máquina en cada comando. No hay una API de callback que configurar: redirige `--format json` a lo que uses para telemetría, o añade una comprobación `exec` para instrumentación personalizada.

## informes JSON

Cada comando usa `--format json`:

```bash
intl-ai fill --format json | jq '{translated, written, unresolved: (.unresolved | length)}'
intl-ai check --format json | jq '.locales.es.invalid'
intl-ai status --format json
```

Errores en stderr tienen la misma estructura: `{"error": {"message": ..., "code": ...}}`.

## Findings

`intl-ai check` reporta hallazgos por clave. Tipos:

| Kind         | Significado                                                                                    |
| ------------ | ------------------------------------------------------------------------------------------ |
| `missing`    | Clave ausente en un archivo de configuración regional de destino.                                                      |
| `stale`      | Valor propio de IA cuyo origen cambió desde el llenado.                                            |
| `modified`   | Valor propiedad de humanos cuyo origen cambió desde la revisión.                                       |
| `unreviewed` | La clave nunca marcada como `reviewed` (o re-etiquetada por `[quality]`).                                |
| `extra`      | Clave presente en un archivo de destino pero no en el origen.                                        |
| `invalid`    | Una entrada `[[checks]]` rechazó el valor (icu, parity, judge, spec, exec o `fail_below`). |

Fallas a nivel de chequeo (una especificación que no cargará, un ejecutable que no se iniciará, un error de proveedor) van al arreglo `errors` del informe y siempre fallan la ejecución. Rejugar un hallazgo desde el caché incremental lo marca como `"cached": true` en la salida JSON.

## Puntuaciones de calidad en el archivo bloqueado

`fill` registra lo que produce en fragmentos de `intl-ai.lock.d/`. Por clave:

- `origin`: `ai` o `human`.
- `reviewed`: se vuelve a `false` automáticamente cuando un humano edita el archivo.
- `quality.scores`: puntuaciones normalizadas por chequeo desde la última ejecución de llenado.
- `quality.unresolved`: claves que fallaron la puerta de validación `[fill].validate` y fueron adoptadas de todos modos.

Supervise la cobertura de revisión en CI con `intl-ai status --format json` y bloquee las fusiones en `intl-ai check --fail-on`.

## Ejecuta comprobaciones como telemetría personalizada

¿Necesitas instrumentación por ejecución dentro del pipeline (publicar en Sentry, emitir una métrica, añadir a un registro)? Escribe una comprobación `exec`: el binario envía una solicitud JSONL por cada lote de claves y espera una respuesta JSONL con los resultados. Es un punto natural para "ejecutar esto en cada ejecución de comprobación" sin bifurcar el pipeline.

```toml
[[checks]]
exec = "./scripts/i18n-telemetry"
timeout_ms = 60000
```

Consulta `examples/checks/` para el protocolo.

## Códigos de salida

- `0`: éxito (o hallazgos solo por debajo de los tipos de `--fail-on`).
- `1`: se encontró un tipo `--fail-on`, falló una puerta configurada o ocurrió un error grave. En CI, un valor distinto de cero siempre significa actuar.

Consulta la [Configuración](/es/guide/configuration/) para `[[checks]]`, `[check]`, `[fill]` y `[quality]`.
