import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { translateBatch } from "./translator";
import { icuProcessor } from "../../adapters/processors/icu";
import type { AIProvider } from "../../ports/provider";

const createTestProvider = (): AIProvider => ({
  id: "test",
  buildRequest({ model, systemPrompt, userPrompt, temperature, modelParams }) {
    return {
      url: "/chat/completions",
      headers: {
        "Content-Type": "application/json",
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
    return {
      content:
        (data as { choices?: Array<{ message?: { content?: string } }> }).choices?.[0]?.message
          ?.content ?? "",
    };
  },
});

const mockOkResponse = (translations: Array<{ key: string; translated: string }>) => ({
  ok: true as const,
  json: async () => ({
    choices: [
      {
        message: {
          content: JSON.stringify({ translations }),
        },
      },
    ],
  }),
});

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
    mockFetch.mockResolvedValueOnce(
      mockOkResponse([{ key: "greeting", translated: "Hola {name}" }]),
    );

    await translateBatch({
      provider: createTestProvider(),
      entries: [{ key: "greeting", source: "Hello {name}" }],
      targetLocale: "es",
      sourceLocale: "en",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
      processor: icuProcessor,
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    const prompt = body.messages[1].content;
    expect(prompt).toContain(icuProcessor.getSyntaxHint());
  });

  it("uses default placeholder hint when no processor", async () => {
    mockFetch.mockResolvedValueOnce(
      mockOkResponse([{ key: "greeting", translated: "Hola {name}" }]),
    );

    await translateBatch({
      provider: createTestProvider(),
      entries: [{ key: "greeting", source: "Hello {name}" }],
      targetLocale: "es",
      sourceLocale: "en",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    const prompt = body.messages[1].content;
    expect(prompt).toContain("Preserve any placeholders like {variable}");
  });

  it("returns successful TranslationResult on valid response", async () => {
    mockFetch.mockResolvedValueOnce(
      mockOkResponse([{ key: "greeting", translated: "Hola {name}" }]),
    );

    const result = await translateBatch({
      provider: createTestProvider(),
      entries: [{ key: "greeting", source: "Hello {name}" }],
      targetLocale: "es",
      sourceLocale: "en",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
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
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

    mockFetch.mockResolvedValueOnce(
      mockOkResponse([{ key: "greeting", translated: "Hola mundo" }]),
    );

    const result = await translateBatch({
      provider: createTestProvider(),
      entries: [{ key: "greeting", source: "Hello {name}" }],
      targetLocale: "es",
      sourceLocale: "en",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
      processor: icuProcessor,
    });

    expect(result[0].success).toBe(false);
    expect(result[0].error).toContain("Validation failed");
    warnSpy.mockRestore();
  });

  it("uses provider.buildRequest and baseURL for fetch", async () => {
    mockFetch.mockResolvedValueOnce(mockOkResponse([{ key: "greeting", translated: "Bonjour" }]));

    await translateBatch({
      provider: createTestProvider(),
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

  it("renders feedback block in the prompt when provided", async () => {
    mockFetch.mockResolvedValueOnce(mockOkResponse([{ key: "greeting", translated: "Bonjour" }]));

    await translateBatch({
      provider: createTestProvider(),
      entries: [{ key: "greeting", source: "Hello" }],
      targetLocale: "fr",
      sourceLocale: "en",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
      feedback: { greeting: "previous attempt was too formal" },
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    const prompt = body.messages[1].content;
    expect(prompt).toContain("rejected by a quality reviewer");
    expect(prompt).toContain("previous attempt was too formal");
  });
});
