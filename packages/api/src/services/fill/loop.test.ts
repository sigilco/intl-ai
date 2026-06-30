import { describe, it, expect, vi } from "vitest";
import { runQualityLoop } from "./loop";
import type { QualityAssessorInstance, QualityResult, TranslationContext } from "../../core/types";

const baseContext = (overrides: Partial<TranslationContext>): TranslationContext => ({
  key: "k",
  source: "Hello",
  translation: "Hola",
  locale: "es",
  sourceHash: "h1",
  origin: "ai",
  model: "gpt-4o-mini",
  provider: "openai",
  ...overrides,
});

describe("runQualityLoop (api)", () => {
  it("returns final translations on the first round when all pass", async () => {
    const judge = vi.fn(async (ctxs: TranslationContext[]) =>
      ctxs.map<QualityResult>((c) => ({
        score: 0.95,
        riskTier: "low",
        needsReview: false,
        assessorName: "test",
      })),
    );
    const refill = vi.fn(async () => []);

    const result = await runQualityLoop({
      initialTranslations: [
        { key: "a", source: "Hello", translated: "Hola" },
        { key: "b", source: "World", translated: "Mundo" },
      ],
      sourceHashByKey: new Map([
        ["a", "h1"],
        ["b", "h2"],
      ]),
      targetLocale: "es",
      provider: "openai",
      model: "gpt-4o-mini",
      judge,
      refill,
      options: { threshold: 0.8, maxRetries: 2 },
    });

    expect(result.attempts).toBe(0);
    expect(result.belowThreshold).toHaveLength(0);
    expect(result.finalByKey.get("a")).toBe("Hola");
    expect(result.finalByKey.get("b")).toBe("Mundo");
    expect(refill).not.toHaveBeenCalled();
  });

  it("refills rejected keys up to maxRetries times", async () => {
    let round = 0;
    const judge = vi.fn(async (ctxs: TranslationContext[]) => {
      round++;
      // reject "a" in round 1, accept in round 2
      return ctxs.map<QualityResult>((c) => {
        const score = c.key === "a" && round === 1 ? 0.3 : 0.95;
        return {
          score,
          riskTier: score < 0.6 ? "high" : "low",
          needsReview: score < 0.8,
          assessorName: "test",
        };
      });
    });

    const refill = vi.fn(async (reqs) =>
      reqs.map((r) => ({
        key: r.key,
        translated: "Refilled",
        success: true,
      })),
    );

    const result = await runQualityLoop({
      initialTranslations: [{ key: "a", source: "Hello", translated: "Initial" }],
      sourceHashByKey: new Map([["a", "h1"]]),
      targetLocale: "es",
      provider: "openai",
      model: "gpt-4o-mini",
      judge,
      refill,
      options: { threshold: 0.8, maxRetries: 2 },
    });

    expect(refill).toHaveBeenCalledTimes(1);
    expect(result.attempts).toBe(1);
    expect(result.finalByKey.get("a")).toBe("Refilled");
    expect(result.belowThreshold).toHaveLength(0);
  });

  it("surfaces keys still below threshold after maxRetries", async () => {
    const judge = vi.fn(async (ctxs: TranslationContext[]) =>
      ctxs.map<QualityResult>((c) => ({
        score: 0.2,
        riskTier: "high",
        needsReview: true,
        reason: "bad translation",
        assessorName: "test",
      })),
    );
    const refill = vi.fn(async (reqs) =>
      reqs.map((r) => ({ key: r.key, translated: "still-bad", success: true })),
    );

    const result = await runQualityLoop({
      initialTranslations: [{ key: "a", source: "Hello", translated: "Initial" }],
      sourceHashByKey: new Map([["a", "h1"]]),
      targetLocale: "es",
      provider: "openai",
      model: "gpt-4o-mini",
      judge,
      refill,
      options: { threshold: 0.8, maxRetries: 1 },
    });

    expect(refill).toHaveBeenCalledTimes(1);
    expect(result.belowThreshold).toHaveLength(1);
    expect(result.belowThreshold[0].key).toBe("a");
    expect(result.belowThreshold[0].quality.score).toBe(0.2);
    // last translation preserved as final
    expect(result.finalByKey.get("a")).toBe("still-bad");
  });

  it("preserves last translation when refill fails", async () => {
    const judge = vi.fn(async (ctxs: TranslationContext[]) =>
      ctxs.map<QualityResult>(() => ({
        score: 0.3,
        riskTier: "high",
        needsReview: true,
        assessorName: "test",
      })),
    );
    const refill = vi.fn(async (reqs) =>
      reqs.map((r) => ({ key: r.key, success: false, error: "refill failed" })),
    );

    const result = await runQualityLoop({
      initialTranslations: [{ key: "a", source: "Hello", translated: "Best-effort" }],
      sourceHashByKey: new Map([["a", "h1"]]),
      targetLocale: "es",
      provider: "openai",
      model: "gpt-4o-mini",
      judge,
      refill,
      options: { threshold: 0.8, maxRetries: 3 },
    });

    expect(result.finalByKey.get("a")).toBe("Best-effort");
    expect(result.belowThreshold).toHaveLength(1);
  });

  it("uses per-key assessor when no judge is provided", async () => {
    const assessor: QualityAssessorInstance = {
      name: "stub",
      async assess(_ctx) {
        return {
          score: 0.9,
          riskTier: "low",
          needsReview: false,
          assessorName: "stub",
        };
      },
    };

    const result = await runQualityLoop({
      initialTranslations: [{ key: "a", source: "Hello", translated: "Hola" }],
      sourceHashByKey: new Map([["a", "h1"]]),
      targetLocale: "es",
      provider: "openai",
      model: "gpt-4o-mini",
      assessor,
      refill: async () => [],
      options: { threshold: 0.8, maxRetries: 1 },
    });

    expect(result.finalByKey.get("a")).toBe("Hola");
  });

  it("throws when neither judge nor assessor is provided", async () => {
    await expect(
      runQualityLoop({
        initialTranslations: [{ key: "a", source: "Hello", translated: "Hola" }],
        sourceHashByKey: new Map([["a", "h1"]]),
        targetLocale: "es",
        provider: "openai",
        model: "gpt-4o-mini",
        refill: async () => [],
        options: { threshold: 0.8, maxRetries: 1 },
      }),
    ).rejects.toThrow(/judge.*assessor/);
  });
});
