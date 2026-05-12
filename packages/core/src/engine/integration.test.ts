import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { translateBatch } from "./translator";
import { icuProcessor } from "../processor/icu";
import { createProcessor } from "../processor/index";
import type { LanguageModel } from "ai";

const createMockModel = (): LanguageModel => {
  return {
    modelId: "test-model",
    provider: "test",
    doGenerate: vi.fn(),
    doStream: vi.fn(),
  } as unknown as LanguageModel;
};

describe("integration: processor → translator → validation pipeline", () => {
  const mockFetch = vi.fn();
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = mockFetch;
    mockFetch.mockReset();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("Test 1: translateBatch with icuProcessor generates prompt containing ICU syntax hint", async () => {
    const model = createMockModel();
    const entries = [{ key: "greeting", source: "Hello {name}" }];

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [{ key: "greeting", translated: "Hola {name}" }],
              }),
            },
          },
        ],
      }),
    });

    await translateBatch({
      model,
      entries,
      targetLocale: "es",
      sourceLocale: "en",
      processor: icuProcessor,
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    const prompt = body.messages[1].content;
    expect(prompt).toContain(icuProcessor.getSyntaxHint());
    expect(prompt).toContain("ICU MessageFormat");
  });

  it("Test 2: translateBatch with custom i18next-style processor generates prompt containing {{variable}} hint", async () => {
    const model = createMockModel();
    const entries = [{ key: "greeting", source: "Hello {{name}}" }];

    const i18nextProcessor = createProcessor({
      name: "i18next",
      extractTokens: (msg: string) => {
        const matches = msg.match(/\{\{(\w+)\}\}/g);
        return matches ? matches.map((m) => m.replace(/\{\{|\}\}/g, "")) : [];
      },
      validate: (source: string, translated: string) => {
        const extract = (msg: string) => {
          const matches = msg.match(/\{\{(\w+)\}\}/g);
          return matches ? matches.map((m) => m.replace(/\{\{|\}\}/g, "")) : [];
        };
        const sourceTokens = extract(source);
        const translatedTokens = extract(translated);
        const missing = sourceTokens.filter((t) => !translatedTokens.includes(t));
        if (missing.length > 0) {
          return { valid: false, errors: [`Missing tokens: ${missing.join(", ")}`] };
        }
        return { valid: true };
      },
      getSyntaxHint: () => "i18next: Use {{variable}} for placeholders.",
    });

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [{ key: "greeting", translated: "Hola {{name}}" }],
              }),
            },
          },
        ],
      }),
    });

    await translateBatch({
      model,
      entries,
      targetLocale: "es",
      sourceLocale: "en",
      processor: i18nextProcessor,
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    const prompt = body.messages[1].content;
    expect(prompt).toContain("i18next: Use {{variable}}");
  });

  it("Test 3: Post-translation validation rejects translations with missing ICU tokens", async () => {
    const model = createMockModel();
    const entries = [{ key: "greeting", source: "Hello {name}" }];

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [{ key: "greeting", translated: "Hola mundo" }],
              }),
            },
          },
        ],
      }),
    });

    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

    const result = await translateBatch({
      model,
      entries,
      targetLocale: "es",
      sourceLocale: "en",
      processor: icuProcessor,
    });

    expect(result).toHaveLength(1);
    expect(result[0].success).toBe(false);
    expect(result[0].error).toContain("Validation failed");

    warnSpy.mockRestore();
  });

  it("Test 4: Post-translation validation accepts translations with correct ICU tokens", async () => {
    const model = createMockModel();
    const entries = [{ key: "greeting", source: "Hello {name}" }];

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [{ key: "greeting", translated: "Hola {name}" }],
              }),
            },
          },
        ],
      }),
    });

    const result = await translateBatch({
      model,
      entries,
      targetLocale: "es",
      sourceLocale: "en",
      processor: icuProcessor,
    });

    expect(result).toHaveLength(1);
    expect(result[0].success).toBe(true);
    expect(result[0].translated).toBe("Hola {name}");
  });

  it("Test 5: No processor = prompt contains {variable} exactly (backward compat)", async () => {
    const model = createMockModel();
    const entries = [{ key: "greeting", source: "Hello {name}" }];

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [{ key: "greeting", translated: "Hola {name}" }],
              }),
            },
          },
        ],
      }),
    });

    await translateBatch({
      model,
      entries,
      targetLocale: "es",
      sourceLocale: "en",
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    const prompt = body.messages[1].content;
    expect(prompt).toContain("Preserve any placeholders like {variable} exactly as they appear.");
  });

  it("Test 6: icuProcessor.validate correctly handles plural messages after translation", async () => {
    const model = createMockModel();
    const entries = [
      { key: "item_count", source: "{count, plural, one {# item} other {# items}}" },
    ];

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [
                  {
                    key: "item_count",
                    translated: "{count, plural, one {# artículo} other {# artículos}}",
                  },
                ],
              }),
            },
          },
        ],
      }),
    });

    const result = await translateBatch({
      model,
      entries,
      targetLocale: "es",
      sourceLocale: "en",
      processor: icuProcessor,
    });

    expect(result).toHaveLength(1);
    expect(result[0].success).toBe(true);
  });
});
