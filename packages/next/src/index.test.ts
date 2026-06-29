import { describe, test, expect, vi } from "vitest";
import { withIntlAi } from "./index";
import type { NextConfig } from "next";

// Mock @intl-ai/unplugin/webpack — the factory returns a webpack plugin object
// with an `apply` method (standard unplugin webpack shape).
vi.mock("@intl-ai/unplugin/webpack", () => {
  const apply = vi.fn();
  const plugin = { apply };
  return {
    default: vi.fn().mockReturnValue(plugin),
  };
});

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

  test("adds intl-ai unplugin to webpack plugins", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrapped = await withIntlAi()(config);

    const mockConfig = { plugins: [] };
    const result = wrapped.webpack!(mockConfig, {} as any);

    expect(result.plugins).toBeDefined();
    expect(Array.isArray(result.plugins)).toBe(true);
    expect(result.plugins.length).toBeGreaterThan(0);

    // unplugin webpack returns an object with an `apply` method
    const intlAiPlugin = result.plugins.find((p: any) => typeof p.apply === "function");
    expect(intlAiPlugin).toBeDefined();
  });

  test("passes debug option to unplugin factory", async () => {
    const config: NextConfig = { reactStrictMode: true };
    const wrapped = await withIntlAi({ debug: true })(config);

    const mockConfig = { plugins: [] };
    wrapped.webpack!(mockConfig, {} as any);

    // Verify the unplugin factory was called with the correct options
    const intlAiUnplugin = (await import("@intl-ai/unplugin/webpack")).default;
    expect(intlAiUnplugin).toHaveBeenCalledWith({ debug: true, quality: false });
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
