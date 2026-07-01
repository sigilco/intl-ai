import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { judgeBatch, createDefaultAssessor } from "./judge";
import type { AIProvider } from "../../ports/provider";
import type { TranslationContext } from "../../core/types";

const createTestProvider = (): AIProvider => ({
  id: "test",
  buildRequest({ model, systemPrompt, userPrompt, temperature, modelParams }) {
    return {
      url: "/chat/completions",
      headers: { "Content-Type": "application/json" },
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

const mockOkResponse = (
  judgements: Array<{ key: string; score: number; reason?: string; errors?: string[] }>,
) => ({
  ok: true as const,
  json: async () => ({
    choices: [{ message: { content: JSON.stringify({ judgements }) } }],
  }),
});

const ctx = (overrides: Partial<TranslationContext>): TranslationContext => ({
  key: "greeting",
  source: "Hello",
  translation: "Hola",
  locale: "es",
  sourceHash: "abc",
  origin: "ai",
  model: "gpt-4o-mini",
  provider: "openai",
  ...overrides,
});

describe("judgeBatch (api)", () => {
  const mockFetch = vi.fn();
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = mockFetch;
    mockFetch.mockReset();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("returns one QualityResult per context, mapped by key", async () => {
    mockFetch.mockResolvedValueOnce(
      mockOkResponse([
        { key: "a", score: 0.9 },
        { key: "b", score: 0.4, reason: "missing negation" },
      ]),
    );

    const results = await judgeBatch({
      provider: createTestProvider(),
      modelId: "test-model",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
      contexts: [ctx({ key: "a" }), ctx({ key: "b" })],
    });

    expect(results).toHaveLength(2);
    expect(results[0]).toMatchObject({ score: 0.9, riskTier: "low", needsReview: false });
    expect(results[1]).toMatchObject({
      score: 0.4,
      riskTier: "high",
      needsReview: true,
      reason: "missing negation",
    });
  });

  it("uses the adversarial system prompt", async () => {
    mockFetch.mockResolvedValueOnce(mockOkResponse([{ key: "a", score: 0.9 }]));

    await judgeBatch({
      provider: createTestProvider(),
      modelId: "test-model",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
      contexts: [ctx({ key: "a" })],
    });

    const call = mockFetch.mock.calls[0];
    const body = JSON.parse(call[1].body);
    expect(body.messages[0].content).toContain("adversarial");
  });

  it("flags missing keys as high risk needs-review", async () => {
    mockFetch.mockResolvedValueOnce(mockOkResponse([{ key: "a", score: 0.9 }]));

    const results = await judgeBatch({
      provider: createTestProvider(),
      modelId: "test-model",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
      contexts: [ctx({ key: "a" }), ctx({ key: "missing" })],
    });

    expect(results[1].needsReview).toBe(true);
    expect(results[1].riskTier).toBe("high");
    expect(results[1].reason).toContain("omitted");
  });

  it("throws on HTTP error", async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
      text: async () => "boom",
      json: async () => ({}),
    } as unknown as Response);

    await expect(
      judgeBatch({
        provider: createTestProvider(),
        modelId: "test-model",
        baseURL: "https://api.test/v1",
        apiKey: "test-key",
        contexts: [ctx({ key: "a" })],
      }),
    ).rejects.toThrow(/HTTP 500/);
  });

  it("createDefaultAssessor wraps judgeBatch and returns a single result", async () => {
    mockFetch.mockResolvedValueOnce(mockOkResponse([{ key: "greeting", score: 0.95 }]));

    const assessor = createDefaultAssessor({
      provider: createTestProvider(),
      modelId: "test-model",
      baseURL: "https://api.test/v1",
      apiKey: "test-key",
    });
    const result = await assessor.assess(ctx({ key: "greeting" }));
    expect(result.assessorName).toBe("default-adversarial-judge");
    expect(result.score).toBe(0.95);
    expect(assessor.name).toBe("default-adversarial-judge");
  });
});
