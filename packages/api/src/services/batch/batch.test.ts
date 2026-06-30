import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { mkdtemp, rm, writeFile, readFile, readdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "pathe";
import { batchedFill } from "./batch";
import { jsonFormat } from "../../adapters/formats/json";
import type { AIProvider } from "../../ports/provider";

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
    const d = data as {
      error?: string;
      choices?: Array<{ message?: { content?: string } }>;
    };
    if (d.error) {
      return { content: JSON.stringify({ error: d.error }) };
    }
    return {
      content: d.choices?.[0]?.message?.content ?? "",
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

describe("batchedFill", () => {
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

  it("writes report-*.json when failures exist and !dryRun", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-batch-report-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ error: "rate limited" }),
      });

      const result = await batchedFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        model: createTestProvider(),
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
        maxRetries: 1,
      });

      expect(result.errors).toBeGreaterThan(0);
      expect(result.failures.length).toBeGreaterThan(0);

      const reportDir = join(dir, ".intl-ai");
      const files = await readdir(reportDir);
      const reportFiles = files.filter((f) => f.startsWith("report-") && f.endsWith(".json"));
      expect(reportFiles.length).toBe(1);

      const report = await readJson(join(reportDir, reportFiles[0]!));
      expect(report.version).toBe(1);
      expect(report.failures).toHaveLength(1);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("does NOT write report when all translations succeed", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-batch-no-report-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );

      const result = await batchedFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        model: createTestProvider(),
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
      });

      expect(result.translated).toBe(1);
      expect(result.errors).toBe(0);
      expect(result.failures).toHaveLength(0);

      const reportDir = join(dir, ".intl-ai");
      await expect(readdir(reportDir)).rejects.toThrow();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("does NOT write report when dryRun is true", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-batch-dryrun-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ error: "rate limited" }),
      });

      const result = await batchedFill(
        {
          defaultLocale: "en",
          locales: ["en", "es"],
          localeDir: dir,
          model: createTestProvider(),
          apiKey: "TEST_API_KEY",
          baseURL: "https://api.test/v1",
          format: jsonFormat,
          maxRetries: 1,
        },
        { dryRun: true },
      );

      expect(result.errors).toBeGreaterThan(0);
      expect(result.failures.length).toBeGreaterThan(0);

      const reportDir = join(dir, ".intl-ai");
      await expect(readdir(reportDir)).rejects.toThrow();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("aggregates failures across locales", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-batch-aggregate-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      for (let i = 0; i < 3; i++) {
        mockFetch.mockResolvedValueOnce({
          ok: true,
          json: async () => ({ error: "rate limited" }),
        });
      }

      const result = await batchedFill(
        {
          defaultLocale: "en",
          locales: ["en", "es", "fr", "de"],
          localeDir: dir,
          model: createTestProvider(),
          apiKey: "TEST_API_KEY",
          baseURL: "https://api.test/v1",
          format: jsonFormat,
          maxRetries: 1,
        },
        { concurrency: 1 },
      );

      expect(result.failures).toHaveLength(3);
      expect(result.errors).toBe(3);
      const failedLocales = new Set(result.failures.map((f) => f.locale));
      expect(failedLocales).toEqual(new Set(["es", "fr", "de"]));
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("runs locales in parallel when concurrency > 1 (smoke test)", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-batch-parallel-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      const timestamps: number[] = [];
      mockFetch.mockImplementation(async () => {
        timestamps.push(performance.now());
        await new Promise((resolve) => setTimeout(resolve, 50));
        return mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] });
      });

      const start = performance.now();
      const result = await batchedFill(
        {
          defaultLocale: "en",
          locales: ["en", "es", "fr", "de"],
          localeDir: dir,
          model: createTestProvider(),
          apiKey: "TEST_API_KEY",
          baseURL: "https://api.test/v1",
          format: jsonFormat,
        },
        { concurrency: 3 },
      );
      const elapsed = performance.now() - start;

      expect(result.translated).toBe(3);
      expect(timestamps).toHaveLength(3);
      // Sequential would take ~150ms; parallel should be ~50ms plus overhead.
      expect(elapsed).toBeLessThan(120);
      // All three fetch calls should start within a short window.
      const spread = Math.max(...timestamps) - Math.min(...timestamps);
      expect(spread).toBeLessThan(30);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("returns same shape as runFill result", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-batch-shape-"));
    try {
      await setupLocaleDir(dir, { "en.json": { greeting: "Hello" } });

      mockFetch.mockResolvedValueOnce(
        mockJsonResponse({ translations: [{ key: "greeting", translated: "Hola" }] }),
      );

      const result = await batchedFill({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: dir,
        model: createTestProvider(),
        apiKey: "TEST_API_KEY",
        baseURL: "https://api.test/v1",
        format: jsonFormat,
      });

      expect(result).toHaveProperty("locales");
      expect(result).toHaveProperty("translated");
      expect(result).toHaveProperty("skipped");
      expect(result).toHaveProperty("errors");
      expect(result).toHaveProperty("needsReview");
      expect(result).toHaveProperty("failures");
      expect(Array.isArray(result.locales)).toBe(true);
      expect(Array.isArray(result.failures)).toBe(true);
      expect(result.locales).toEqual(["es"]);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
