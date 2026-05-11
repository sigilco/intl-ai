---
title: AI Model Setup
---

# AI Model Setup

Configure your AI provider for translation generation. Translation tasks are structured and instruction-following — budget models handle them well, so there's no need to reach for flagship pricing.

## Supported Providers

### 1. LM Studio (Recommended for Local Development)

LM Studio allows you to run AI models locally without cloud dependencies. This is ideal for development, testing, and privacy-sensitive work.

Download from [lmstudio.ai](https://lmstudio.ai), load a model (recommended: Qwen 3 4B), and start the local server.

#### Configuration

```typescript
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";

const lmstudio = createOpenAICompatible({
  name: "lmstudio",
  baseURL: "http://127.0.0.1:1234/v1",
});

export default {
  model: lmstudio("qwen3-4b"),
};
```

### 2. OpenAI

Get an API key from [platform.openai.com](https://platform.openai.com) and set `OPENAI_API_KEY` in your environment.

```typescript
import { openai } from "@ai-sdk/openai";

export default {
  model: openai("gpt-4.1-nano"),
};
```

**Recommended Models:**

| Model | Input / 1M tokens | Output / 1M tokens | Context |
|---|---|---|---|
| `gpt-4.1-nano` | $0.10 | $0.40 | 1M |
| `gpt-4.1-mini` | $0.40 | $1.60 | 1M |
| `gpt-4.1` | $2.00 | $8.00 | 1M |

### 3. Anthropic Claude

Get an API key from [console.anthropic.com](https://console.anthropic.com) and set `ANTHROPIC_API_KEY` in your environment.

```typescript
import { anthropic } from "@ai-sdk/anthropic";

export default {
  model: anthropic("claude-haiku-4-5-20250414"),
};
```

**Recommended Models:**

| Model | Input / 1M tokens | Output / 1M tokens | Context |
|---|---|---|---|
| `claude-haiku-4-5` | $1.00 | $5.00 | 200K |
| `claude-sonnet-4-5` | $3.00 | $15.00 | 200K |

### 4. Google Gemini

Set up a Google AI project and get an API key from [aistudio.google.com](https://aistudio.google.com). Set `GOOGLE_GENERATIVE_AI_API_KEY` in your environment.

```typescript
import { google } from "@ai-sdk/google";

export default {
  model: google("gemini-2.5-flash"),
};
```

**Recommended Models:**

| Model | Input / 1M tokens | Output / 1M tokens | Context |
|---|---|---|---|
| `gemini-2.5-flash` | $0.15 | $0.60 | 1M |
| `gemini-2.5-pro` | $1.25 | $10.00 | 1M |

> **Best value for translations:** `gemini-2.5-flash` — cheapest capable model with a 1M context window. `gpt-4.1-nano` is a close second if you're already in the OpenAI ecosystem.

## Context Window Requirements

All models must support a **minimum of 16,000 tokens** context window. Every model listed above exceeds this by a wide margin.

If you encounter connection issues, verify your API key is set and your provider's server is accessible.
