---
title: Plugins de formato
description: Enseña a intl-ai a leer y escribir cualquier formato de archivo de locale con un plugin externo.
---

# Plugins de formato

intl-ai incluye formatos de locale JSON y YAML integrados. Una entrada `[[formats]]` registra un formato personalizado respaldado por un programa externo, por lo que cualquier formato de archivo (Android `strings.xml`, ARB, PO, `.strings`) puede funcionar con `intl-ai fill` y `intl-ai check` en cualquier lenguaje de implementación, sin enlazar Rust.

## Registrar un formato

```toml
format = "xml"

[[formats]]
name = "xml"
exec = "python3"
args = ["tools/strings_xml.py"]
extension = "xml"
```

| Clave                | Obligatorio | Propósito                                                                  |
| ------------------ | -------- | ------------------------------------------------------------------------ |
| `name`             | sí      | Nombre del registro; `format = "<name>"` lo selecciona. No debe ser `json`/`yaml` |
| `exec`             | sí      | Programa a lanzar (un proceso por operación de lectura/escritura)                         |
| `args`             | no       | Argumentos pasados a `exec`                                                    |
| `extension`        | sí      | Extensión canónica emitida para nuevos archivos (`xml` -> `fr.xml`)             |
| `extensions`       | no       | Extensiones adicionales que el formato reclama en disco                               |
| `cwd`              | no       | Directorio de trabajo del plugin (por defecto: el directorio del archivo de configuración)          |
| `timeout_ms`       | no       | Presupuesto de tiempo de pared por operación (por defecto `60000`)                               |
| `max_stdout_bytes` | no       | Límite de stdout almacenado en búfer (por defecto 10 MiB; las respuestas incluyen archivos completos)        |

`format` acepta nombres integrados (`json`, `yaml`) o un nombre registrado `[[formats]]`. Un nombre desconocido falla la validación de configuración con la lista de nombres conocidos.

::: warning
`[[formats]] exec` solo puede aparecer en el `intl-ai.toml` raíz, nunca en un archivo cargado mediante `extends`. Este es el mismo límite de confianza que `[[checks]] exec`: una configuración base proporcionada por una dependencia no debe registrar programas para ejecutar en tu máquina.
:::

## Cómo se resuelven los archivos

La resolución es inalterada: la extensión propia de un archivo existente gana, y el `format` configurado solo emite nuevos archivos. Un proyecto que mezcle `en.json`, `fr.yaml` y `de.xml` en un solo `locale_dir` lee cada archivo con el formato que reclamó su extensión.

## Protocolo v1

El plugin es una transformación pura entre el texto del archivo y el árbol del corpus (un objeto JSON de clave anidada -> cadena). intl-ai posee toda la E/S de archivos: lee el archivo, entrega el texto al plugin y escribe el texto devuelto de forma atómica. El plugin nunca toca el sistema de archivos.

Un proceso hijo por operación. stdin recibe exactamente una línea de solicitud JSONL; stdout debe producir exactamente una línea de respuesta JSONL. Ambas partes son estrictas con `v`: intl-ai envía `v: 1` y exige `v: 1` de vuelta, y un plugin que reciba un `v` que no admite debe responder con una respuesta `error`.

### Leer

Solicitud:

```json
{ "v": 1, "op": "read", "format": "xml", "path": "/abs/locales/fr.xml", "content": "<file text>" }
```

Respuesta:

```json
{ "v": 1, "data": { "greeting": "Salut!", "nav": { "home": "Accueil" } } }
```

`data` debe ser un objeto JSON. `path` es informativo (úsalo en mensajes de error); los plugins no deben abrirlo.

### Escribir

Solicitud:

```json
{
  "v": 1,
  "op": "write",
  "format": "xml",
  "path": "/abs/locales/fr.xml",
  "data": { "greeting": "Salut!" }
}
```

Respuesta:

```json
{ "v": 1, "content": "<resources>...</resources>\n" }
```

`content` es el texto completo del archivo; intl-ai lo escribe literalmente (omitindo la escritura cuando los bytes no cambian).

### Errores y códigos de salida

Los fallos a nivel de formato (entrada no analizable, construcciones no soportadas) se informan como una respuesta con un campo `error` y código de salida 0:

```json
{ "v": 1, "error": "parse:unclosed tag on line 3" }
```

`error` es texto libre; anteponerle un código estable (`parse:`, `write:`) es la convención. Una salida distinta de cero, un tiempo de espera agotado o una línea de respuesta malformada/ausente significa que el plugin falló y la operación falla de la misma manera, con el final de stderr adjunto.

El texto del archivo es UTF-8; los formatos binarios están fuera de alcance para v1.

## Ejemplo

[`examples/formats/strings-xml`](https://github.com/sigilco/intl-ai/tree/develop/examples/formats/strings-xml) es un plugin de referencia: un solo script de la biblioteca estándar de Python que lee y escribe un subconjunto mínimo de `strings.xml` de Android. Dado que las claves de strings.xml son planas, aplana el corpus anidado en nombres con puntos al escribir (`nav.home` -> `<string name="nav.home">`); un plugin puede en cambio devolver objetos anidados para preservar la agrupación.

## Escribir tu propio

1. Lee una línea JSON de stdin, escribe una línea JSON a stdout.
2. Mantén `v: 1` en cada respuesta.
3. Nunca leer o escribir `path`; transforma solo `content`/`data`.
4. Informa fallos de formato como `{"v": 1, "error": "..."}` con salida 0; reserva las salidas distintas de cero para fallos críticos.
5. Añade la entrada a `intl-ai.toml`, luego verifica con `intl-ai check` (lee todos los archivos de locale) y `intl-ai fill` (emite los que faltan).
