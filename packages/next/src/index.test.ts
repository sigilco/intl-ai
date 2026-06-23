import { describe, test, expect, vi, beforeEach } from "vitest";
import { withIntlAi } from "./index";
import type { NextConfig } from "next";

// Mock the inlined config loader
vi.mock("./config", () => ({
  loadConfig: vi.fn().mockResolvedValue({
    defaultLocale: "en",
    locales: ["en", "es", "fr"],
    localeDir: "./locales",
    model: "gpt-4",
    glossary: {},
    maxRetries: 3,
  }),
}));

// Mock @intl-ai/api for runFill
vi.mock("@intl-ai/api", () => ({
  runFill: vi.fn().mockResolvedValue({
    locales: ["es", "fr"],
    translated: 0,
    skipped: 0,
    errors: 0,
  }),
}));

describe("withIntlAi", () => {
  test("wraps object config", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrapped = await withIntlAi()(config);

    expect(wrapped).toBeDefined();
    expect(wrapped.webpack).toBeDefined();
    expect(typeof wrapped.webpack).toBe("function");
    expect(wrapped.reactStrictMode).toBe(true);
  });

  test("wraps sync function config", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrappedFn = withIntlAi()(() => config);
    const wrapped = await wrappedFn;

    expect(wrapped).toBeDefined();
    expect(wrapped.webpack).toBeDefined();
    expect(typeof wrapped.webpack).toBe("function");
    expect(wrapped.reactStrictMode).toBe(true);
  });

  test("wraps async function config", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrappedFn = withIntlAi()(async () => config);
    const wrapped = await wrappedFn;

    expect(wrapped).toBeDefined();
    expect(wrapped.webpack).toBeDefined();
    expect(typeof wrapped.webpack).toBe("function");
    expect(wrapped.reactStrictMode).toBe(true);
  });

  test("preserves existing webpack config", async () => {
    let called = false;
    const customWebpack = (cfg: any) => {
      called = true;
      return { ...cfg, custom: true };
    };
    const config: NextConfig = {
      webpack: customWebpack,
    };
    const wrapped = await withIntlAi()(config);

    expect(wrapped.webpack).toBeDefined();
    expect(typeof wrapped.webpack).toBe("function");

    const mockConfig = { plugins: [] };
    const result = wrapped.webpack!(mockConfig, {} as any);

    expect(called).toBe(true);
    expect(result.custom).toBe(true);
  });

  test("adds intl-ai webpack plugin", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrapped = await withIntlAi()(config);

    const mockConfig = { plugins: [] };
    const result = wrapped.webpack!(mockConfig, {} as any);

    expect(result.plugins).toBeDefined();
    expect(Array.isArray(result.plugins)).toBe(true);
    expect(result.plugins.length).toBeGreaterThan(0);

    const intlAiPlugin = result.plugins.find((p: any) => p.name === "intl-ai-webpack-plugin");
    expect(intlAiPlugin).toBeDefined();
  });

  test("passes options to webpack plugin", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrapped = await withIntlAi({ debug: true })(config);

    const mockConfig = { plugins: [] };
    const result = wrapped.webpack!(mockConfig, {} as any);

    const intlAiPlugin = result.plugins.find((p: any) => p.name === "intl-ai-webpack-plugin");
    expect(intlAiPlugin).toBeDefined();
    expect(intlAiPlugin.options.debug).toBe(true);
  });

  test("preserves all NextConfig properties", async () => {
    const config: NextConfig = {
      reactStrictMode: true,
      experimental: {
        caseSensitiveRoutes: true,
      },
    };
    const wrapped = await withIntlAi()(config);

    expect(wrapped.reactStrictMode).toBe(true);
    expect(wrapped.experimental).toBeDefined();
    expect(wrapped.experimental?.caseSensitiveRoutes).toBe(true);
    expect(wrapped.webpack).toBeDefined();
  });
});
