---
title: AI model setup
---

# AI model setup

Translation is structured and instruction-following. Budget models handle it well, so you do not need a flagship model.

## Pick a provider

Three paths work: local, cloud, or an aggregator.

### Local: LM Studio

[LM Studio](https://lmstudio.ai) runs models locally. This is ideal for development, testing, and privacy-sensitive work.

Download LM Studio, load any model that fits your hardware, and start the local server.

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

export default {
  provider: resolveProvider("openai"),
  model: "qwen3.5-4b-instruct",
  apiKey: "lm-studio",
  baseURL: "http://127.0.0.1:1234/v1",
  defaultLocale: "en",
  locales: ["en", "de", "es", "fr"],
  localeDir: "./locales",
};
```

### Cloud: OpenAI-compatible providers

Any provider with an OpenAI-compatible endpoint works the same way: set the provider ID, model name, API key, and base URL.

- **OpenAI**: set `OPENAI_API_KEY`. See [platform.openai.com](https://platform.openai.com).
- **Anthropic**: set `ANTHROPIC_API_KEY`. See [console.anthropic.com](https://console.anthropic.com).
- **Google**: set `GOOGLE_GENERATIVE_AI_API_KEY`. See [aistudio.google.com](https://aistudio.google.com).
- **Azure OpenAI**, **Cohere**, **Mistral**, and others: use the matching base URL and a custom AIProvider if the API shape differs from OpenAI.

Example with OpenAI:

```typescript
export default {
  provider: "openai",
  model: "gpt-4o-mini",
  apiKey: "${OPENAI_API_KEY}",
  baseURL: "https://api.openai.com/v1",
};
```

Example with Anthropic:

```typescript
export default {
  provider: "anthropic",
  model: "claude-3-5-haiku-latest",
  apiKey: "${ANTHROPIC_API_KEY}",
  baseURL: "https://api.anthropic.com/v1",
  modelParams: { max_tokens: 1024 },
};
```

### Aggregator: OpenRouter

[OpenRouter](https://openrouter.ai) gives you one API key for many providers and a free tier.
A stable free model at the time of writing is `google/gemini-2.0-flash-exp:free`.
If it stops working, check OpenRouter's free model list and update this single reference.

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

const openrouter = resolveProvider("openai");

export default {
  provider: openrouter,
  model: "google/gemini-2.0-flash-exp:free",
  apiKey: "${OPENROUTER_API_KEY}",
  baseURL: "https://openrouter.ai/api/v1",
};
```

## Context window

Use a model with a minimum context window of 16,000 tokens. Every provider listed above exceeds this.

## Troubleshooting

If translation fails, check:

- The API key environment variable is set.
- The provider ID in your config matches a supported provider.
- The provider's server is reachable from your machine.
- Your model identifier matches the provider's documentation.
