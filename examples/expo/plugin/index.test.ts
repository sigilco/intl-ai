import { describe, expect, it } from "vitest";
import { withIntlAi } from "./index";

describe("intl-ai expo plugin", () => {
  it("returns a config plugin function", () => {
    expect(typeof withIntlAi).toBe("function");
  });

  it("wraps a base Expo config and attaches mods", () => {
    const base = { name: "TestApp", slug: "test-app" };
    const wrapped = withIntlAi(base, { configPath: "intl-ai.config.json" });

    expect(wrapped.name).toBe("TestApp");
    expect(wrapped.slug).toBe("test-app");
    expect(wrapped.mods).toBeDefined();
  });

  it("uses the provided config path in the dangerous mod", () => {
    const wrapped = withIntlAi(
      { name: "App", slug: "app" },
      {
        configPath: "config/intl-ai.json",
        verbose: true,
      },
    );

    expect(wrapped.mods?.ios).toBeDefined();
  });
});
