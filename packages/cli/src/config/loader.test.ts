import { describe, test, expect, beforeEach, afterEach } from "vitest";
import { writeFileSync, mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "pathe";
import { loadConfig, validateJsonConfig } from "./loader";

describe("config loader", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = join(
      tmpdir(),
      `intl-ai-loader-test-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    );
    mkdirSync(tempDir, { recursive: true });
  });

  afterEach(() => {
    rmSync(tempDir, { recursive: true, force: true });
  });

  test("loads and converts a valid JSON config", async () => {
    const configPath = join(tempDir, "intl-ai.config.json");
    writeFileSync(
      configPath,
      JSON.stringify({
        defaultLocale: "en",
        locales: ["en", "fr"],
        localeDir: "./locales",
        provider: "gpt-4o-mini",
        model: "gpt-4o-mini",
        apiKey: "sk-test",
        baseURL: "https://api.example.com/v1",
        maxRetries: 2,
        processor: "icu",
      }),
    );

    const config = await loadConfig(configPath, { validate: false });

    expect(config.defaultLocale).toBe("en");
    expect(config.locales).toEqual(["en", "fr"]);
    expect(config.localeDir).toBe(join(tempDir, "locales"));
    expect(config.maxRetries).toBe(2);
    expect(config.processor?.name).toBe("icu");

    expect(config.model).toBe("gpt-4o-mini");
    expect(config.apiKey).toBe("sk-test");
    expect(config.baseURL).toBe("https://api.example.com/v1");
  });

  test("loads a TypeScript config via jiti", async () => {
    const configPath = join(tempDir, "intl-ai.config.ts");
    writeFileSync(
      configPath,
      `export default {
        defaultLocale: "en",
        locales: ["en", "fr"],
        localeDir: "./locales",
        model: { modelId: "gpt-4o-mini", config: { apiKey: "x", baseURL: "https://x" } },
      };`,
      "utf-8",
    );

    const config = await loadConfig(configPath, { validate: false });

    expect(config.defaultLocale).toBe("en");
    expect(config.locales).toEqual(["en", "fr"]);
    expect(config.localeDir).toBe(join(tempDir, "locales"));
  });

  test("throws on missing file", async () => {
    await expect(loadConfig(join(tempDir, "missing.json"))).rejects.toThrow(
      "Config file not found",
    );
  });

  test("throws on invalid JSON", async () => {
    const configPath = join(tempDir, "bad.json");
    writeFileSync(configPath, "{ not json");
    await expect(loadConfig(configPath)).rejects.toThrow("Invalid JSON");
  });

  test("throws on schema validation failure", async () => {
    const configPath = join(tempDir, "invalid.json");
    writeFileSync(
      configPath,
      JSON.stringify({
        defaultLocale: "en",
        // missing required locales, localeDir, model, apiKey
      }),
    );
    await expect(loadConfig(configPath)).rejects.toThrow(/Config validation failed/);
  });

  test("validateJsonConfig accepts valid config", () => {
    expect(() =>
      validateJsonConfig({
        defaultLocale: "en",
        locales: ["en", "fr"],
        localeDir: "./locales",
        model: "gpt-4o-mini",
        apiKey: "sk-test",
      }),
    ).not.toThrow();
  });

  test("throws on empty defaultLocale", () => {
    expect(() =>
      validateJsonConfig({
        defaultLocale: "",
        locales: ["en", "fr"],
        localeDir: "./locales",
        model: "gpt-4o-mini",
        apiKey: "sk-test",
      }),
    ).toThrow(/Config validation failed/);
  });

  test("throws on invalid processor value", () => {
    expect(() =>
      validateJsonConfig({
        defaultLocale: "en",
        locales: ["en", "fr"],
        localeDir: "./locales",
        model: "gpt-4o-mini",
        apiKey: "sk-test",
        processor: "unknown-processor",
      }),
    ).toThrow(/Config validation failed/);
  });

  test("throws on missing apiKey and model", () => {
    expect(() =>
      validateJsonConfig({
        defaultLocale: "en",
        locales: ["en", "fr"],
        localeDir: "./locales",
      }),
    ).toThrow(/Config validation failed/);
  });
});
