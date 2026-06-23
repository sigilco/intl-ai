import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { translateBatch } from "./translator";
import { icuProcessor } from "../processor/icu";

const createMockModel = () => {
  return {
    modelId: "test-model",
    provider: "test",
    config: {
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
    },
  };
};

describe("translateBatch (api)", () => {
  const mockFetch = vi.fn();
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = mockFetch;
    mockFetch.mockReset();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("uses icuProcessor syntax hint in prompt", async () => {
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
  });

  it("uses default placeholder hint when no processor", async () => {
    const model = createMockModel();
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
      entries: [{ key: "greeting", source: "Hello {name}" }],
      targetLocale: "es",
      sourceLocale: "en",
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    const prompt = body.messages[1].content;
    expect(prompt).toContain("Preserve any placeholders like {variable}");
  });

  it("returns successful TranslationResult on valid response", async () => {
    const model = createMockModel();
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
      entries: [{ key: "greeting", source: "Hello {name}" }],
      targetLocale: "es",
      sourceLocale: "en",
      processor: icuProcessor,
    });

    expect(result).toHaveLength(1);
    expect(result[0]).toEqual({
      key: "greeting",
      translated: "Hola {name}",
      success: true,
    });
  });

  it("rejects translations missing required tokens (validation)", async () => {
    const model = createMockModel();
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

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

    const result = await translateBatch({
      model,
      entries: [{ key: "greeting", source: "Hello {name}" }],
      targetLocale: "es",
      sourceLocale: "en",
      processor: icuProcessor,
    });

    expect(result[0].success).toBe(false);
    expect(result[0].error).toContain("Validation failed");
    warnSpy.mockRestore();
  });

  it("supports baseURL/apiKey overrides on the call", async () => {
    const model = createMockModel();
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [{ key: "greeting", translated: "Bonjour" }],
              }),
            },
          },
        ],
      }),
    });

    await translateBatch({
      model,
      entries: [{ key: "greeting", source: "Hello" }],
      targetLocale: "fr",
      sourceLocale: "en",
      baseURL: "https://override.example/v1",
      apiKey: "override-key",
    });

    const call = mockFetch.mock.calls[0];
    expect(call[0]).toBe("https://override.example/v1/chat/completions");
    expect(call[1].headers.Authorization).toBe("Bearer override-key");
  });
});
