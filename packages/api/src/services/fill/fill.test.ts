import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { mkdtemp, rm, writeFile, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "pathe";
import { runFill, IntlAiQualityError } from "./fill";
import { jsonFormat } from "../../adapters/formats/json";
import type { AIProvider } from "../../ports/provider";
import type { QualityAssessorInstance } from "../../core/types";

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

const mockJsonResponse = (body: unknown) => ({
  ok: true as const,
  json: async () => ({ choices: [{ message: { content: JSON.stringify(body) } }] }),
});

async function setupLocaleDir(
  dir: string,
  files: Record<string, Record<string, string>>,
): Promise<void> {
  for (const [name, contents] of Object.entries(files)) {
    await writeFile(join(dir, name), JSON.stringify(contents, null, 2), "utf-8");
  }
}

async function readJson(path: string): Promise<Record<string, unknown>> {
  return JSON.parse(await readFile(path, "utf-8"));
}

describe("runFill (api) — quality loop", () => {
  const mockFetch = vi.fn();
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = mockFetch;
    mockFetch.mockReset();
    process.env.TEST_API_KEY = "test-key";
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("model field from config reaches HTTP request body", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-roundtrip-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      const requestedModels: string[] = [];
      const capturingProvider: AIProvider = {
        id: "capture-test",
        buildRequest({ model, systemPrompt, userPrompt }) {
          requestedModels.push(model);
          return {
            url: "/chat/completions",
            headers: { "Content-Type": "application/json" },
            body: {
              model,
              messages: [
                { role: "system", content: systemPrompt },
                { role: "user", content: userPrompt },
              ],
            },
          };
        },
        parseResponse(data: unknown) {
          return {
            content: JSON.stringify({ translations: [{ key: "greeting", translated: "Hola" }] }),
          };
        },
      };

      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );

      await runFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        provider: capturingProvider,
        model: "my-custom-model-v9",
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
      });

      expect(requestedModels).toEqual(["my-custom-model-v9"]);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("locks correct sourceHash for flat dot-notation keys (regression: v0.3.0 hashed empty string)", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-flatkeys-"));
    try {
      // Flat dot-notation keys: "nav.home" is a literal top-level key, not a nested path.
      await setupLocaleDir(dir, {
        "en.json": { "nav.home": "Home", "stash.title": "Stash title" },
      });

      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({
          translations: [
            { key: "nav.home", translated: "Inicio" },
            { key: "stash.title", translated: "Titulo del stash" },
          ],
        }),
      );

      const result = await runFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        provider: createTestProvider(),
        model: "test-model",
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
      });

      expect(result.translated).toBe(2);

      const lockfile = await readJson(join(dir, "intl-ai.lock.json"));
      const EMPTY_SHA1 = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
      for (const key of ["nav.home", "stash.title"]) {
        const entry = lockfile.entries[`es||${key}`] as Record<string, unknown>;
        expect(entry, `missing lockfile entry for ${key}`).toBeDefined();
        expect(entry.sourceHash, `${key} hashed the empty string (v0.3.0 bug)`).not.toBe(
          EMPTY_SHA1,
        );
      }

      // Hashes must match SHA-1 of the actual source values.
      const { createHash } = await import("node:crypto");
      expect(lockfile.entries["es||nav.home"].sourceHash).toBe(
        createHash("sha1").update("Home").digest("hex"),
      );
      expect(lockfile.entries["es||stash.title"].sourceHash).toBe(
        createHash("sha1").update("Stash title").digest("hex"),
      );

      // And the follow-up fill must be a no-op (nothing stale), proving staleness detection works.
      const again = await runFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        provider: createTestProvider(),
        model: "test-model",
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
      });
      expect(again.translated).toBe(0);
      expect(again.skipped).toBe(2);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("writes quality records to lockfile when assessor accepts all keys", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-quality-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      // Round 1: translate
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );
      // Round 2 (quality): judge
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ judgements: [{ key: "greeting", score: 0.95 }] }),
      );

      const result = await runFill(
        {
          defaultLocale: "en",
          locales: ["en", "es"],
          localeDir: dir,
          provider: createTestProvider(),
          model: "test-model",
          apiKey: "TEST_API_KEY",
          baseURL: "https://api.test/v1",
          format: jsonFormat,
        },
        { quality: { threshold: 0.8, maxRetries: 2 } },
      );

      expect(result.translated).toBe(1);
      expect(result.needsReview).toBe(0);
      const lockfile = await readJson(join(dir, "intl-ai.lock.json"));
      const entry = lockfile.entries["es||greeting"] as Record<string, unknown>;
      expect(entry.translated).toBe("Hola");
      expect(entry.quality).toBeDefined();
      const quality = entry.quality as Record<string, unknown>;
      expect(quality.score).toBe(0.95);
      expect(quality.riskTier).toBe("low");
      expect(quality.needsReview).toBe(false);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("refills below-threshold keys up to maxRetries", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-refill-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      // 1. Initial translate
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );
      // 2. Judge rejects
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({
          judgements: [{ key: "greeting", score: 0.4, reason: "too formal" }],
        }),
      );
      // 3. Refill
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Qué tal" }] }),
      );
      // 4. Judge accepts
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ judgements: [{ key: "greeting", score: 0.95 }] }),
      );

      const result = await runFill(
        {
          defaultLocale: "en",
          locales: ["en", "es"],
          localeDir: dir,
          provider: createTestProvider(),
          model: "test-model",
          apiKey: "TEST_API_KEY",
          baseURL: "https://api.test/v1",
          format: jsonFormat,
        },
        { quality: { threshold: 0.8, maxRetries: 2 } },
      );

      expect(result.translated).toBe(1);
      expect(result.needsReview).toBe(0);

      const es = await readJson(join(dir, "es.json"));
      expect(es.greeting).toBe("Qué tal");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("throws IntlAiQualityError when failOnLowQuality and below threshold after retries", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-throw-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      // Round 0: initial translate
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );
      // Round 0 judge rejects
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ judgements: [{ key: "greeting", score: 0.2 }] }),
      );
      // Refill attempt 1
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Refilled-1" }] }),
      );
      // Refill attempt 1 judge rejects
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ judgements: [{ key: "greeting", score: 0.2 }] }),
      );
      // Refill attempt 2
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Refilled-2" }] }),
      );
      // Refill attempt 2 judge rejects (final round, maxRetries=2 exhausted)
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ judgements: [{ key: "greeting", score: 0.2 }] }),
      );

      await expect(
        runFill(
          {
            defaultLocale: "en",
            locales: ["en", "es"],
            localeDir: dir,
            provider: createTestProvider(),
            model: "test-model",
            apiKey: "TEST_API_KEY",
            baseURL: "https://api.test/v1",
            format: jsonFormat,
          },
          { quality: { threshold: 0.8, maxRetries: 2, failOnLowQuality: true } },
        ),
      ).rejects.toThrow(IntlAiQualityError);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("records below-threshold without throwing when failOnLowQuality is false", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-warn-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      // 1. translate
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );
      // 2. judge (rejects)
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ judgements: [{ key: "greeting", score: 0.2 }] }),
      );
      // 3. refill (still bad)
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Refilled" }] }),
      );
      // 4. judge (rejects again)
      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ judgements: [{ key: "greeting", score: 0.2 }] }),
      );

      const result = await runFill(
        {
          defaultLocale: "en",
          locales: ["en", "es"],
          localeDir: dir,
          provider: createTestProvider(),
          model: "test-model",
          apiKey: "TEST_API_KEY",
          baseURL: "https://api.test/v1",
          format: jsonFormat,
        },
        { quality: { threshold: 0.8, maxRetries: 1 } },
      );

      expect(result.translated).toBe(1);
      expect(result.needsReview).toBe(1);
      const es = await readJson(join(dir, "es.json"));
      expect(es.greeting).toBe("Refilled");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("uses a custom QualityAssessorInstance when configured", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-custom-assessor-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );

      const assessor: QualityAssessorInstance = {
        name: "unit-test-assessor",
        async assess() {
          return {
            score: 0.99,
            riskTier: "low",
            needsReview: false,
            assessorName: "unit-test-assessor",
          };
        },
      };

      const result = await runFill(
        {
          defaultLocale: "en",
          locales: ["en", "es"],
          localeDir: dir,
          provider: createTestProvider(),
          model: "test-model",
          apiKey: "TEST_API_KEY",
          baseURL: "https://api.test/v1",
          format: jsonFormat,
          quality: { threshold: 0.8, maxRetries: 1, assessor },
        },
        { quality: {} },
      );

      expect(result.translated).toBe(1);
      // judgeBatch must NOT have been called — only the per-key assessor
      expect(mockFetch.mock.calls.length).toBe(1);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});

describe("runFill (api) — batching", () => {
  const mockFetch = vi.fn();
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = mockFetch;
    mockFetch.mockReset();
    process.env.TEST_API_KEY = "test-key";
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("sends all entries in one request even when batchSize is set", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-batch-"));
    try {
      await setupLocaleDir(dir, { "en.json": { a: "A", b: "B", c: "C" } });

      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({
          translations: [
            { key: "a", translated: "Á" },
            { key: "b", translated: "Bé" },
            { key: "c", translated: "Cé" },
          ],
        }),
      );

      const result = await runFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        provider: createTestProvider(),
        model: "test-model",
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
        batchSize: 2,
      });

      expect(result.translated).toBe(3);
      expect(mockFetch).toHaveBeenCalledTimes(1);
      const es = await readJson(join(dir, "es.json"));
      expect(es.a).toBe("Á");
      expect(es.b).toBe("Bé");
      expect(es.c).toBe("Cé");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("sends all entries in one request when batchSize is omitted", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-nobatch-"));
    try {
      await setupLocaleDir(dir, { "en.json": { a: "A", b: "B" } });

      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({
          translations: [
            { key: "a", translated: "Á" },
            { key: "b", translated: "Bé" },
          ],
        }),
      );

      const result = await runFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        provider: createTestProvider(),
        model: "test-model",
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
      });

      expect(result.translated).toBe(2);
      expect(mockFetch).toHaveBeenCalledTimes(1);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("bare subtag localeInstructions key applies to both en-GB and en-US, and --force isolates locales", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-fill-locale-instructions-"));
    try {
      await setupLocaleDir(dir, {
        "en.json": { greeting: "Hello" },
        "en-GB.json": { greeting: "Hello" },
        "en-US.json": { greeting: "Hello" },
      });

      const seenPrompts: string[] = [];
      const capturingProvider: AIProvider = {
        id: "capture-instructions",
        buildRequest({ model, systemPrompt, userPrompt, temperature, modelParams }) {
          seenPrompts.push(systemPrompt);
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
        parseResponse: createTestProvider().parseResponse,
      };

      mockFetch.mockResolvedValue(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "'Ello" }] }),
      );

      const baseConfig = {
        defaultLocale: "en",
        locales: ["en", "en-GB", "en-US"],
        localeDir: dir,
        provider: capturingProvider,
        model: "test-model",
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
        localeInstructions: { en: "Use informal register." },
      };

      // en-GB.json and en-US.json already have all keys, so a first pass with
      // no --force should not retranslate either — force is needed to exercise
      // both locales through the instruction-bearing path.
      await runFill(baseConfig, { force: true });

      expect(seenPrompts).toHaveLength(2);
      expect(seenPrompts.every((p) => p.includes("Use informal register."))).toBe(true);

      // Now confirm --force with an explicit locale only touches that locale.
      mockFetch.mockClear();
      seenPrompts.length = 0;
      await writeFile(
        join(dir, "en-GB.json"),
        JSON.stringify({ greeting: "Hello" }, null, 2),
        "utf-8",
      );
      await writeFile(
        join(dir, "en-US.json"),
        JSON.stringify({ greeting: "Untouched" }, null, 2),
        "utf-8",
      );

      await runFill(baseConfig, { force: true, locale: "en-GB" });

      expect(mockFetch).toHaveBeenCalledTimes(1);
      const enUs = await readJson(join(dir, "en-US.json"));
      expect(enUs.greeting).toBe("Untouched");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
