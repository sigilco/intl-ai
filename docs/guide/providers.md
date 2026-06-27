---
title: Providers
---

# Providers

intl-ai translates locale keys by calling an AI model over HTTP. Every provider encapsulates how requests are shaped for a specific API and how responses are parsed back into translations. You can use a built-in provider, or write your own for any compatible API.

## How it works

Each provider implements the `AIProvider` interface from `@intl-ai/api`:

```typescript
interface AIProvider {
  readonly id: string;
  buildRequest(opts: {
    model: string;
    systemPrompt: string;
    userPrompt: string;
    temperature: number;
    modelParams?: Record<string, unknown>;
  }): {
    url: string;
    headers: Record<string, string>;
    body: Record<string, unknown>;
  };
  parseResponse(data: unknown): { content: string };
}
```

At build time, intl-ai:

1. Passes the system prompt (translation instructions) and user prompt (keys + source text) into `buildRequest`.
2. Sends the resulting HTTP request to `baseURL + url`.
3. Feeds the parsed JSON response through `parseResponse` to extract the translated string.

This means the translation logic itself is decoupled from any specific vendor. Swap providers without changing your locale files or config structure.

## Built-in providers

intl-ai ships with two providers: `openai` and `anthropic`.

### OpenAI

Provider ID: `openai`

- Uses the `/chat/completions` endpoint.
- Passes `Authorization: Bearer ${apiKey}` in the request header.
- Sends the system prompt and user prompt as separate message roles.
- Enables JSON Schema structured output to guarantee the response shape.
- The `apiKey` placeholder in the header is resolved from environment variables at runtime via `resolveApiKey`.

### Anthropic

Provider ID: `anthropic`

- Uses the `/messages` endpoint.
- Passes `X-Api-Key: ${apiKey}` and `anthropic-version: 2023-06-01` in the request headers.
- Sends the system prompt as a `system` array (Anthropic's preferred format) and the user prompt as a `user` message.
- Defaults `max_tokens` to `1024` when no `modelParams.max_tokens` is provided.

## Registry resolution

Use `resolveProvider()` from `@intl-ai/api/internal` to turn a config value into a concrete provider instance:

```typescript
import { resolveProvider } from "@intl-ai/api/internal";

// By ID string — returns the matching built-in provider
const openai = resolveProvider("openai");
const anthropic = resolveProvider("anthropic");

// By object — passes through directly (custom provider)
const custom = resolveProvider(myProvider);
```

When you pass a string, `resolveProvider` looks it up in the built-in registry. If the ID is not recognized, it throws an error listing the available providers. When you pass an `AIProvider` object, it returns it unchanged.

In your config, the `provider` field accepts either form:

```typescript
// intl-ai.config.ts — string form
export default {
  provider: "anthropic",
  model: "claude-sonnet-4-20250514",
  apiKey: "${ANTHROPIC_API_KEY}",
  baseURL: "https://api.anthropic.com/v1",
  defaultLocale: "en",
  locales: ["en", "es"],
  localeDir: "./locales",
};
```

```typescript
// intl-ai.config.ts — custom provider form
import { resolveProvider } from "@intl-ai/api/internal";

export default {
  provider: resolveProvider(myProvider),
  model: "my-model",
  apiKey: "${MY_API_KEY}",
  baseURL: "https://my-api.example.com/v1",
  defaultLocale: "en",
  locales: ["en", "es"],
  localeDir: "./locales",
};
```

## Writing a custom provider

You only need to implement `id`, `buildRequest`, and `parseResponse`. Here is a minimal provider for an OpenAI-compatible API (for example, a self-hosted vLLM or LiteLLM instance):

```typescript
import type { AIProvider } from "@intl-ai/api";

const myProvider: AIProvider = {
  id: "my-provider",

  buildRequest({ model, systemPrompt, userPrompt, temperature, modelParams }) {
    return {
      url: "/chat/completions",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer ${apiKey}",
      },
      body: {
        model,
        messages: [
          { role: "system", content: systemPrompt },
          { role: "user", content: userPrompt },
        ],
        temperature,
        ...modelParams,
      },
    };
  },

  parseResponse(data: unknown) {
    // Adapt this to your API's response shape
    const body = data as { choices?: Array<{ message?: { content?: string } }> };
    return {
      content: body.choices?.[0]?.message?.content ?? "",
    };
  },
};
```

Then pass it to your config:

```typescript
import { resolveProvider } from "@intl-ai/api/internal";
import { myProvider } from "./my-provider";

export default {
  provider: resolveProvider(myProvider),
  model: "my-model-name",
  apiKey: "${MY_PROVIDER_API_KEY}",
  baseURL: "https://my-provider.example.com/v1",
  defaultLocale: "en",
  locales: ["en", "es"],
  localeDir: "./locales",
};
```

### The buildRequest contract

`buildRequest` receives an object with:

- `model` — the model name from config.
- `systemPrompt` — translation instructions including glossary and locale context.
- `userPrompt` — the actual keys and source strings to translate.
- `temperature` — sampling temperature from config.
- `modelParams` — optional extra parameters forwarded from config.

It must return:

- `url` — the path appended to `baseURL`.
- `headers` — HTTP headers. Use `"${apiKey}"` as a placeholder and intl-ai resolves it from environment variables.
- `body` — the JSON request payload.

### The parseResponse contract

`parseResponse` receives the parsed JSON body of the API response. It must return `{ content: string }` where `content` is the raw translation text (typically a JSON string that intl-ai parses further).

## Provider quirks

A few things to keep in mind when configuring providers:

**Anthropic requires `max_tokens`.** The Anthropic API rejects requests without a `max_tokens` field. The built-in provider defaults to `1024`, but you can override it with `modelParams`:

```json
{
  "provider": "anthropic",
  "modelParams": { "max_tokens": 4096 }
}
```

**Base URLs differ per provider.** OpenAI uses `https://api.openai.com/v1`. Anthropic uses `https://api.anthropic.com/v1`. If you are using a proxy or self-hosted endpoint, set `baseURL` in your config to match your endpoint.

**API key header format.** OpenAI expects `Authorization: Bearer <key>`. Anthropic expects `X-Api-Key: <key>`. Custom providers should follow the format their API expects. The `${apiKey}` placeholder in the header value is automatically replaced by intl-ai at runtime.

**Structured output.** The OpenAI provider enables JSON Schema structured output in `response_format`. This guarantees the model returns valid JSON matching the translation schema. If your custom provider's API does not support structured output, rely on prompt engineering instead, and ensure `parseResponse` handles the response shape accordingly.

**modelParams are spread into the body.** Anything you put in `modelParams` is merged into the request body via the spread operator. This means provider-specific fields like `top_p`, `frequency_penalty`, or `max_tokens` go there. Be careful not to override fields the provider already sets (like `messages` or `model`).
