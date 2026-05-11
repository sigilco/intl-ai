import { describe, it, expect } from "vitest";
import { icuProcessor, passthroughProcessor, createProcessor } from "../index";

describe("processor exports", () => {
  describe("icuProcessor", () => {
    it("should be importable from @intl-ai/core", () => {
      expect(icuProcessor).toBeDefined();
    });

    it("should have a name property", () => {
      expect(typeof icuProcessor.name).toBe("string");
    });

    it("should have an extractTokens method", () => {
      expect(typeof icuProcessor.extractTokens).toBe("function");
    });

    it("should have a validate method", () => {
      expect(typeof icuProcessor.validate).toBe("function");
    });

    it("should have a getSyntaxHint method", () => {
      expect(typeof icuProcessor.getSyntaxHint).toBe("function");
    });

    it("should have name 'icu'", () => {
      expect(icuProcessor.name).toBe("icu");
    });
  });

  describe("passthroughProcessor", () => {
    it("should be importable from @intl-ai/core", () => {
      expect(passthroughProcessor).toBeDefined();
    });

    it("should have a name property", () => {
      expect(typeof passthroughProcessor.name).toBe("string");
    });

    it("should have an extractTokens method", () => {
      expect(typeof passthroughProcessor.extractTokens).toBe("function");
    });

    it("should have a validate method", () => {
      expect(typeof passthroughProcessor.validate).toBe("function");
    });

    it("should have a getSyntaxHint method", () => {
      expect(typeof passthroughProcessor.getSyntaxHint).toBe("function");
    });

    it("should have name 'passthrough'", () => {
      expect(passthroughProcessor.name).toBe("passthrough");
    });
  });

  describe("createProcessor", () => {
    it("should be importable from @intl-ai/core", () => {
      expect(typeof createProcessor).toBe("function");
    });

    it("should return an object with all 4 methods", () => {
      const processor = createProcessor({ name: "test" });
      expect(typeof processor.name).toBe("string");
      expect(typeof processor.extractTokens).toBe("function");
      expect(typeof processor.validate).toBe("function");
      expect(typeof processor.getSyntaxHint).toBe("function");
    });

    it("should use provided name", () => {
      const processor = createProcessor({ name: "custom-name" });
      expect(processor.name).toBe("custom-name");
    });

    it("should fill defaults when overrides not provided", () => {
      const processor = createProcessor({});
      expect(processor.name).toBe("custom-processor");
      expect(processor.extractTokens("")).toEqual([]);
      expect(processor.validate("a", "b")).toEqual({ valid: true });
      expect(processor.getSyntaxHint()).toBe("");
    });
  });
});
