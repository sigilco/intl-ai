import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { translateBatch } from "./translator";
import { icuProcessor } from "../processor/icu";
import type { LanguageModel } from "ai";

const createMockModel = (): LanguageModel => {
  return {
    modelId: "test-model",
    provider: "test",
    doGenerate: vi.fn(),
    doStream: vi.fn(),
  } as unknown as LanguageModel;
};

describe("translateBatch", () => {
  const mockFetch = vi.fn();
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = mockFetch;
    mockFetch.mockReset();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("Test 1: translateBatch with icuProcessor - prompt contains ICU syntax hint", async () => {
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
    expect(prompt).not.toContain(
      "Preserve any placeholders like {variable} exactly as they appear.",
    );
  });

  it("Test 2: translateBatch without processor - prompt contains exactly the old {variable} text", async () => {
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
    expect(prompt).not.toContain("ICU MessageFormat");
  });

  it("Test 3: Post-translation validation passes (mock fetch returning valid ICU translation)", async () => {
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
    expect(result[0]).toEqual({
      key: "greeting",
      translated: "Hola {name}",
      success: true,
    });
  });

  it("Test 4: Post-translation validation fails (mock fetch returning translation missing ICU tokens)", async () => {
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
    expect(result[0].error).toContain("Missing tokens");
    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining("Translation validation failed for key 'greeting'"),
    );

    warnSpy.mockRestore();
  });

  it("Test 5: Backward compat - no processor = identical prompt string", async () => {
    const model = createMockModel();
    const entries = [
      { key: "greeting", source: "Hello {name}" },
      { key: "farewell", source: "Goodbye {name}" },
    ];

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        choices: [
          {
            message: {
              content: JSON.stringify({
                translations: [
                  { key: "greeting", translated: "Hola {name}" },
                  { key: "farewell", translated: "Adios {name}" },
                ],
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
    expect(prompt).toContain("Hello {name}");
    expect(prompt).toContain("Goodbye {name}");
  });
});
