---
title: Providers
description: intl-ai provider reference. HTTP (OpenAI-compatible), command (local CLI agents), and replay transports.
---

# Providers

intl-ai translates locale keys by calling an AI model. The `[provider]` block in `intl-ai.toml` picks the transport; everything else (prompt contract, batching, retries, the judge) is shared.

## `kind = "http"`

An OpenAI-compatible chat-completions endpoint. The broadest surface: OpenAI, OpenRouter, LM Studio, Ollama, vLLM, Azure OpenAI, and most hosted providers expose this shape.

```toml
[provider]
kind = "http"
provider = "openai"                 # wire-shape preset (default "openai")
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"

[provider.model_params]
temperature = 0.2                   # extra request-body params, spread last
```

| Field          | Description                                                                                    |
| -------------- | ---------------------------------------------------------------------------------------------- |
| `provider`     | Wire-shape preset name; defaults to `openai`.                                                  |
| `model`        | Model identifier sent in the request body.                                                     |
| `api_key`      | Sent as `Authorization: Bearer <key>`. Supports `${env:VAR}` and `${file:PATH}` interpolation. |
| `base_url`     | Endpoint override; defaults to the provider's canonical URL.                                   |
| `model_params` | Extra request-body params, spread last so they win.                                            |

Requests retry with backoff up to `max_retries` (default 3).

## `kind = "command"`

Runs a local CLI agent and reads translations back from its stdout. Use this for providers with a different wire shape, agent CLIs you already have (an `agent` preset names a known one), or custom tooling.

```toml
[provider]
kind = "command"
command = "my-translator"
args = ["--json"]
prompt_via = "stdin"        # or "argv" to append the prompt as the last arg
cwd = "."
timeout_ms = 300000
```

| Field              | Description                                                      |
| ------------------ | ---------------------------------------------------------------- |
| `command`          | Free-form command line. Root config only, never via `extends`.   |
| `agent`            | Preset name from the built-in agent table (kebab-case).          |
| `args`             | Extra arguments passed to the command.                           |
| `prompt_via`       | `stdin` (default) or `argv`.                                     |
| `cwd`              | Working directory, resolved against the config file's directory. |
| `timeout_ms`       | Wall-clock budget before SIGTERM, then SIGKILL (default 300000). |
| `max_stdout_bytes` | Buffered stdout cap.                                             |

Because `command` can spawn arbitrary programs, it is refused in `extends`-ed config files: data from a dependency must not execute code.

## `kind = "replay"`

A cassette JSON of canned translations, for tests and demos. No network, no credentials, deterministic output.

```toml
[provider]
kind = "replay"
file = "cassette.json"
```

The cassette maps `{ "<locale>": { "<source value>": "<translation>" } }` and resolves against the config file's directory.

## The prompt contract

Every transport answers the same prompt: the source locale's flattened keys plus instructions, glossary terms, and the placeholder contract (`processor = "icu"` mandates ICU MessageFormat). The response must be a JSON object of translations. `fill` parses it strictly; the `judge` check scores quality through the same transport.

See [Configuration](/guide/configuration/) for the full schema.
