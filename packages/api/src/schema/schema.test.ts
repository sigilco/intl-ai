import { describe, it, expect } from "vitest";
import { getIntlAiSchema, jsonConfigToIntlAiConfig, INTL_AI_SCHEMA_URL } from "../internal";

describe("schema", () => {
  it("exposes a valid JSON Schema object", () => {
    const schema = getIntlAiSchema();
    expect(schema).toBeTypeOf("object");
    expect((schema as Record<string, unknown>).$id).toBe(INTL_AI_SCHEMA_URL);
    expect((schema as Record<string, unknown>).type).toBe("object");
  });

  it("requires defaultLocale, locales, localeDir, provider, apiKey, model", () => {
    const schema = getIntlAiSchema() as { required: string[] };
    expect(schema.required).toEqual(
      expect.arrayContaining(["defaultLocale", "locales", "localeDir", "provider", "apiKey", "model"]),
    );
  });

  it("parity: JSON Schema required fields match Zod schema", () => {
    const jsonSchema = getIntlAiSchema() as {
      required?: string[];
      properties?: Record<string, unknown>;
    };

    // The known required fields in the Zod schema
    const expectedRequired = ["defaultLocale", "locales", "localeDir", "provider", "model", "apiKey"];

    expect(jsonSchema.required).toEqual(expect.arrayContaining(expectedRequired));
    expect(jsonSchema.properties).toHaveProperty("model");
    expect(jsonSchema.properties).toHaveProperty("provider");
  });

  it("jsonConfigToIntlAiConfig produces a valid runtime config", () => {
    const cfg = jsonConfigToIntlAiConfig({
      defaultLocale: "en",
      locales: ["en", "es"],
      localeDir: "./locales",
      provider: "openai",
      model: "gpt-4o-mini",
      apiKey: "test-key",
    });
    expect(cfg.defaultLocale).toBe("en");
    expect(cfg.locales).toEqual(["en", "es"]);
    expect(cfg.maxRetries).toBe(3);
    expect(cfg.provider).toBe("openai");
    expect(cfg.model).toBe("gpt-4o-mini");
  });

  it("processor: 'icu' attaches the ICU processor", () => {
    const cfg = jsonConfigToIntlAiConfig({
      defaultLocale: "en",
      locales: ["en", "es"],
      localeDir: "./locales",
      provider: "openai",
      model: "gpt-4o-mini",
      apiKey: "k",
      processor: "icu",
    });
    expect(cfg.processor?.name).toBe("icu");
  });

  it("quality: { threshold, maxRetries } propagates to runtime config", () => {
    const cfg = jsonConfigToIntlAiConfig({
      defaultLocale: "en",
      locales: ["en", "es"],
      localeDir: "./locales",
      provider: "openai",
      model: "gpt-4o-mini",
      apiKey: "k",
      quality: { threshold: 0.7, maxRetries: 3 },
    });
    expect(cfg.quality).toEqual({ threshold: 0.7, maxRetries: 3 });
  });
});
