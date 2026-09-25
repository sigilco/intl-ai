# Jev check recipe

An `[[checks]] exec` example that scores translations with TypeSafe's
Jev API, keeping Jev out of the core binary — the check speaks the
versioned JSONL protocol, so any eval service can slot in the same way.

## Wiring

```toml
[[checks]]
exec = "python3"
args = ["examples/checks/jev/jev_check.py"]
```

`intl-ai check` then reports a Jev score below the threshold as an
`invalid` finding per key.

## Environment

| Variable           | Purpose                                        |
| ------------------ | ---------------------------------------------- |
| `TYPESAFE_API_KEY` | Jev API credential (required)                  |
| `JEV_API_URL`      | Score endpoint (default in the script)         |
| `JEV_THRESHOLD`    | Flag findings below this score (default `0.8`) |

## Protocol

The script implements exec protocol `v: 1`: one request line on stdin
(`items` = the locale's `{key, source, target}` batch), one response
line on stdout (`findings` or `error`). To write your own check, keep
the same request/response pair in any language — a shell script with
`jq` works too.
