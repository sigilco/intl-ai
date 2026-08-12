import { describe, it, expect } from "vitest";
import { translateBatch } from "./translator";
import { openaiProvider } from "../../adapters/providers/openai";
import { icuProcessor } from "../../adapters/processors/icu";

/**
 * Hits the real OpenRouter API. Local-only: run via `pnpm test:integration`,
 * never part of `pnpm test` (excluded in vitest.config.ts) or CI (no secrets
 * there, and CI also short-circuits below as a second guard).
 */
const hasCredentials = Boolean(process.env.OPENROUTER_API_KEY && process.env.OPENROUTER_MODEL_ID);

describe.skipIf(!hasCredentials || Boolean(process.env.CI))("translateBatch (OpenRouter, live)", () => {
  it("translates a batch through the real HTTP provider", async () => {
    const result = await translateBatch({
      provider: openaiProvider,
      modelId: process.env.OPENROUTER_MODEL_ID,
      baseURL: "https://openrouter.ai/api/v1",
      apiKey: "${OPENROUTER_API_KEY}",
      processor: icuProcessor,
      entries: [
        { key: "greeting", source: "Hello {name}, welcome back." },
        { key: "farewell", source: "See you soon." },
      ],
      targetLocale: "es",
      sourceLocale: "en",
    });

    expect(result).toHaveLength(2);
    for (const entry of result) {
      expect(entry.success).toBe(true);
      expect(entry.translated).toBeTruthy();
    }
    expect(result.find((r) => r.key === "greeting")?.translated).toContain("{name}");
  }, 60_000);
});
