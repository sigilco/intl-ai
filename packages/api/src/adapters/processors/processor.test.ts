import { describe, it, expect } from "vitest";
import { icuProcessor } from "./icu";
import { passthroughProcessor, createProcessor } from "./index";

describe("processor/icu (api)", () => {
  it("extracts simple placeholder", () => {
    expect(icuProcessor.extractTokens("Hello {name}")).toEqual(["name"]);
  });

  it("validates missing tokens", () => {
    const r = icuProcessor.validate("Hello {name}", "Bonjour");
    expect(r.valid).toBe(false);
    expect(r.errors?.[0]).toContain("name");
  });

  it("validates correct translations", () => {
    const r = icuProcessor.validate("Hello {name}", "Bonjour {name}");
    expect(r.valid).toBe(true);
  });
});

describe("processor/index (api)", () => {
  it("passthrough processor always valid", () => {
    expect(passthroughProcessor.validate("x", "y").valid).toBe(true);
  });

  it("createProcessor fills missing methods", () => {
    const p = createProcessor({ name: "test" });
    expect(p.name).toBe("test");
    expect(p.extractTokens("")).toEqual([]);
    expect(p.validate("a", "b").valid).toBe(true);
  });
});
