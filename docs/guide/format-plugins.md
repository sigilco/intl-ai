---
title: Format plugins
description: Teach intl-ai to read and write any locale file format with an external plugin.
---

# Format plugins

intl-ai ships with JSON and YAML locale formats builtin. A `[[formats]]` entry registers a custom format backed by an external program, so any file format (Android `strings.xml`, ARB, PO, `.strings`) can work with `intl-ai fill` and `intl-ai check` in any implementation language, without linking Rust.

## Registering a format

```toml
format = "xml"

[[formats]]
name = "xml"
exec = "python3"
args = ["tools/strings_xml.py"]
extension = "xml"
```

| Key                | Required | Purpose                                                                  |
| ------------------ | -------- | ------------------------------------------------------------------------ |
| `name`             | yes      | Registry name; `format = "<name>"` selects it. Must not be `json`/`yaml` |
| `exec`             | yes      | Program to spawn (one process per read/write op)                         |
| `args`             | no       | Argv passed to `exec`                                                    |
| `extension`        | yes      | Canonical extension minted for new files (`xml` -> `fr.xml`)             |
| `extensions`       | no       | Extra extensions the format claims on disk                               |
| `cwd`              | no       | Plugin working directory (default: the config file's directory)          |
| `timeout_ms`       | no       | Wall-clock budget per op (default `60000`)                               |
| `max_stdout_bytes` | no       | Buffered stdout cap (default 10 MiB; responses carry whole files)        |

`format` accepts builtin names (`json`, `yaml`) or a registered `[[formats]]` name. An unknown name fails config validation with the list of known names.

::: warning
`[[formats]] exec` can only appear in the root `intl-ai.toml`, never in a file pulled in via `extends`. This is the same trust boundary as `[[checks]] exec`: a base config shipped by a dependency must not register programs to run on your machine.
:::

## How files resolve

Resolution is unchanged: an existing file's own extension wins, and the configured `format` only mints new files. A project mixing `en.json`, `fr.yaml`, and `de.xml` in one `locale_dir` reads each file with the format that claimed its extension.

## Protocol v1

The plugin is a pure transform between file text and the corpus tree (a nested key -> string JSON object). intl-ai owns all file IO: it reads the file, hands the text to the plugin, and writes the returned text atomically. The plugin never touches the filesystem.

One child process per operation. stdin gets exactly one JSONL request line; stdout must produce exactly one JSONL response line. Both sides are strict about `v`: intl-ai sends `v: 1` and requires `v: 1` back, and a plugin receiving a `v` it does not support should answer with an `error` response.

### Read

Request:

```json
{ "v": 1, "op": "read", "format": "xml", "path": "/abs/locales/fr.xml", "content": "<file text>" }
```

Response:

```json
{ "v": 1, "data": { "greeting": "Salut!", "nav": { "home": "Accueil" } } }
```

`data` must be a JSON object. `path` is informational (use it in error messages); plugins must not open it.

### Write

Request:

```json
{
  "v": 1,
  "op": "write",
  "format": "xml",
  "path": "/abs/locales/fr.xml",
  "data": { "greeting": "Salut!" }
}
```

Response:

```json
{ "v": 1, "content": "<resources>...</resources>\n" }
```

`content` is the complete file text; intl-ai writes it verbatim (skipping the write when bytes are unchanged).

### Errors and exit codes

Format-level failures (unparseable input, unsupported constructs) are reported as a response with an `error` field and exit code 0:

```json
{ "v": 1, "error": "parse:unclosed tag on line 3" }
```

`error` is free text; prefixing it with a stable code (`parse:`, `write:`) is the convention. A nonzero exit, a timeout, or a malformed/absent response line means the plugin crashed and fails the operation the same way, with the stderr tail attached.

File text is UTF-8; binary formats are out of scope for v1.

## Example

[`examples/formats/strings-xml`](https://github.com/sigilco/intl-ai/tree/develop/examples/formats/strings-xml) is a reference plugin: a single Python stdlib script that reads and writes a minimal subset of Android `strings.xml`. Because strings.xml keys are flat, it flattens the nested corpus into dotted names on write (`nav.home` -> `<string name="nav.home">`); a plugin may instead return nested objects to preserve grouping.

## Writing your own

1. Read one JSON line from stdin, write one JSON line to stdout.
2. Keep `v: 1` on every response.
3. Never read or write `path`; transform `content`/`data` only.
4. Report format failures as `{"v": 1, "error": "..."}` with exit 0; reserve nonzero exits for crashes.
5. Add the entry to `intl-ai.toml`, then verify with `intl-ai check` (reads every locale file) and `intl-ai fill` (mints missing ones).
