---
title: AI model setup
description: Configure any AI model for intl-ai. Any OpenAI-compatible endpoint, a local CLI agent, or a replay cassette.
---

# AI model setup

Translation is structured and instruction-following. Budget models handle it well, so you do not need a flagship model.

## Pick a provider

Three paths work: a local server, a cloud endpoint, or an aggregator. All three use `kind = "http"` against an OpenAI-compatible chat-completions API.

### Local: LM Studio or Ollama

[LM Studio](https://lmstudio.ai) and [Ollama](https://ollama.com) run models locally and expose an OpenAI-compatible server. This is ideal for development, testing, and privacy-sensitive work.

```toml
[provider]
kind = "http"
base_url = "http://127.0.0.1:1234/v1"
model = "your-local-model"
api_key = "lm-studio"
```

### Cloud: OpenAI-compatible providers

Any provider with an OpenAI-compatible endpoint works the same way: set `base_url`, `model`, and `api_key`.

```toml
[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

Anthropic, Google, Azure OpenAI, Cohere, Mistral, and others all expose OpenAI-compatible surfaces (first-party or via a proxy). Point `base_url` at yours and set `model` accordingly. For an API with a different wire shape, use the `command` provider to shell out to a local agent instead; see [Providers](/guide/providers/).

### Aggregator: OpenRouter

[OpenRouter](https://openrouter.ai) gives you one API key for many providers and a free tier.
A stable free model at the time of writing is `google/gemini-2.0-flash-exp:free`.
If it stops working, check OpenRouter's free model list and update this single reference.

```toml
[provider]
kind = "http"
base_url = "https://openrouter.ai/api/v1"
model = "google/gemini-2.0-flash-exp:free"
api_key = "${env:OPENROUTER_API_KEY}"
```

## Tuning the request

`model_params` spreads extra fields into the request body last, so your params win over the defaults:

```toml
[provider.model_params]
temperature = 0.2
max_tokens = 1024
```

## Context window

Use a model with a minimum context window of 16,000 tokens. Every provider listed above exceeds this.

## Troubleshooting

If translation fails, check:

- The `api_key` interpolation resolves (`intl-ai config validate` shows it masked).
- The provider's server is reachable from your machine (`base_url`).
- Your `model` identifier matches the provider's documentation.
