import { describe, it, expect } from "vitest";
import { getIntlAiSchema, jsonConfigToIntlAiConfig, INTL_AI_SCHEMA_URL } from "../internal";
import { IntlAiJsonConfigSchema } from "./json-config";
import { IntlAiConfigSchema, defineConfig } from "../types";

interface JsonSchemaObjectArm {
  type: "object";
  required?: string[];
  properties?: Record<string, unknown>;
  additionalProperties?: boolean;
}

function armFor(schema: unknown, kind: "http" | "agent"): JsonSchemaObjectArm {
  const oneOf = (schema as { oneOf: JsonSchemaObjectArm[] }).oneOf;
  const arm = oneOf.find(
    (a) => (a.properties?.kind as { const?: string } | undefined)?.const === kind,
  );
  if (!arm) throw new Error(`no ${kind} arm in generated schema`);
  return arm;
}

describe("schema", () => {
  it("exposes a valid JSON Schema object", () => {
    const schema = getIntlAiSchema();
    expect(schema).toBeTypeOf("object");
    expect((schema as Record<string, unknown>).$id).toBe(INTL_AI_SCHEMA_URL);
    expect((schema as Record<string, unknown>).oneOf).toBeInstanceOf(Array);
  });

  it("http arm requires defaultLocale, locales, localeDir, provider, apiKey, model", () => {
    const arm = armFor(getIntlAiSchema(), "http");
    expect(arm.required).toEqual(
      expect.arrayContaining([
        "defaultLocale",
        "locales",
        "localeDir",
        "provider",
        "apiKey",
        "model",
      ]),
    );
  });

  it("agent arm requires defaultLocale, locales, localeDir, agent and rejects apiKey/model", () => {
    const arm = armFor(getIntlAiSchema(), "agent");
    expect(arm.required).toEqual(
      expect.arrayContaining(["defaultLocale", "locales", "localeDir", "agent"]),
    );
    expect(arm.properties).not.toHaveProperty("apiKey");
    expect(arm.properties).not.toHaveProperty("model");
    expect(arm.properties).not.toHaveProperty("command");
  });

  it("both arms preserve additionalProperties: false (union does not leak fields across kinds)", () => {
    const schema = getIntlAiSchema();
    expect(armFor(schema, "http").additionalProperties).toBe(false);
    expect(armFor(schema, "agent").additionalProperties).toBe(false);
  });

  it("parity: JSON Schema required fields match Zod schema", () => {
    const arm = armFor(getIntlAiSchema(), "http");

    // The known required fields in the Zod schema
    const expectedRequired = [
      "defaultLocale",
      "locales",
      "localeDir",
      "provider",
      "model",
      "apiKey",
    ];

    expect(arm.required).toEqual(expect.arrayContaining(expectedRequired));
    expect(arm.properties).toHaveProperty("model");
    expect(arm.properties).toHaveProperty("provider");
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

  it("localeInstructions propagates to runtime config", () => {
    const cfg = jsonConfigToIntlAiConfig({
      defaultLocale: "en",
      locales: ["en-GB", "en-US"],
      localeDir: "./locales",
      provider: "openai",
      model: "gpt-4o-mini",
      apiKey: "k",
      localeInstructions: { en: "Use British spelling for en-GB, American for en-US." },
    });
    expect(cfg.localeInstructions).toEqual({
      en: "Use British spelling for en-GB, American for en-US.",
    });
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

  describe("JSON config parsing", () => {
    it("preserves the JSON Schema meta-key $schema verbatim", () => {
      const r = IntlAiJsonConfigSchema.safeParse({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "./locales",
        provider: "openai",
        model: "gpt-4o-mini",
        apiKey: "k",
        $schema: "https://www.schemastore.org/intl-ai.json",
      });
      expect(r.success).toBe(true);
      if (r.success) {
        expect(r.data.$schema).toBe("https://www.schemastore.org/intl-ai.json");
      }
    });

    it("accepts the actual examples/expo config (which uses $schema)", () => {
      // Regression: .strict() used to reject $schema. Reproduces the user-reported
      // failure mode against the on-disk examples/ config.
      const cfg = {
        $schema: "https://www.schemastore.org/intl-ai.json",
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "locales",
        provider: "openai",
        model: "gpt-4o-mini",
        apiKey: "${OPENAI_API_KEY}",
        baseURL: "https://api.openai.com/v1",
        maxRetries: 3,
      };
      const r = IntlAiJsonConfigSchema.safeParse(cfg);
      expect(r.success).toBe(true);
      if (r.success) {
        expect(r.data.model).toBe("gpt-4o-mini");
        expect(r.data.$schema).toBe("https://www.schemastore.org/intl-ai.json");
      }
    });

    it("still rejects unrelated unknown keys (strict)", () => {
      const r = IntlAiJsonConfigSchema.safeParse({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "./locales",
        provider: "openai",
        model: "gpt-4o-mini",
        apiKey: "k",
        bogusKey: 1,
      });
      expect(r.success).toBe(false);
    });
  });

  describe("kind union", () => {
    it("an existing http config with no kind still validates", () => {
      const r = IntlAiJsonConfigSchema.safeParse({
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "./locales",
        provider: "openai",
        model: "gpt-4o-mini",
        apiKey: "k",
      });
      expect(r.success).toBe(true);
      if (r.success) {
        expect((r.data as { kind: string }).kind).toBe("http");
      }
    });

    it("an agent config with apiKey is rejected", () => {
      const r = IntlAiJsonConfigSchema.safeParse({
        kind: "agent",
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "./locales",
        agent: "claude-code",
        apiKey: "k",
      });
      expect(r.success).toBe(false);
    });

    it("an agent config with command is rejected from JSON (preset name only)", () => {
      const r = IntlAiJsonConfigSchema.safeParse({
        kind: "agent",
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "./locales",
        agent: "claude-code",
        command: "claude",
      });
      expect(r.success).toBe(false);
    });

    it("a valid agent config with a preset name only validates", () => {
      const r = IntlAiJsonConfigSchema.safeParse({
        kind: "agent",
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "./locales",
        agent: "claude-code",
      });
      expect(r.success).toBe(true);
    });

    it("command/args are accepted from the TS runtime config (not JSON)", () => {
      const cfg = defineConfig({
        kind: "agent",
        defaultLocale: "en",
        locales: ["en", "es"],
        localeDir: "./locales",
        command: "my-agent",
        args: ["--quiet"],
      });
      const r = IntlAiConfigSchema.safeParse(cfg);
      expect(r.success).toBe(true);
    });
  });
});
