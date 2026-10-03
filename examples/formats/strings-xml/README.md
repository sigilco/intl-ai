# Android strings.xml format plugin

A `[[formats]] exec` example that reads and writes a minimal subset of
Android `strings.xml`, keeping the format out of the core binary.
The plugin speaks the versioned JSONL protocol, so a format in any
language can slot in the same way.

## Wiring

```toml
format = "xml"

[[formats]]
name = "xml"
exec = "python3"
args = ["examples/formats/strings-xml/strings_xml.py"]
extension = "xml"
```

`intl-ai fill` then mints `locales/<locale>.xml` files, and `intl-ai
check` reads existing ones. An existing `.json`/`.yaml` file still wins
over `format` for its own path.

## Shape

strings.xml is flat, so the plugin flattens the nested corpus into
dotted names on write (`nav.home` -> `<string name="nav.home">`) and
returns names verbatim on read. Only `<string>` elements round-trip;
`<plurals>` and `<string-array>` are skipped on read. This is a
reference plugin, not a full Android adapter.

## Protocol

The script implements format protocol `v: 1`: one request line on stdin
(`content` = file text for reads, `data` = corpus object for writes),
one response line on stdout (`data`, `content`, or `error`). Format
errors exit 0 with an `error` field; a nonzero exit means the plugin
crashed. See `docs/guide/format-plugins.md` for the full contract.
