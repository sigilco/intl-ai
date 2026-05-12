import { describe, it, expect } from "vitest";
import { icuProcessor } from "../index";

describe("icuProcessor", () => {
  describe("extractTokens", () => {
    it("extracts simple argument", () => {
      expect(icuProcessor.extractTokens("{name}")).toEqual(["name"]);
    });

    it("extracts plural argument", () => {
      expect(icuProcessor.extractTokens("{count, plural, one {# item} other {# items}}")).toEqual([
        "count",
      ]);
    });

    it("extracts select argument", () => {
      expect(
        icuProcessor.extractTokens("{gender, select, male {He} female {She} other {They}}"),
      ).toEqual(["gender"]);
    });

    it("extracts selectordinal argument", () => {
      expect(icuProcessor.extractTokens("{rank, selectordinal, one {#st} other {#th}}")).toEqual([
        "rank",
      ]);
    });

    it("extracts number argument", () => {
      expect(icuProcessor.extractTokens("{price, number}")).toEqual(["price"]);
    });

    it("extracts date argument", () => {
      expect(icuProcessor.extractTokens("{date, date}")).toEqual(["date"]);
    });

    it("extracts time argument", () => {
      expect(icuProcessor.extractTokens("{time, time}")).toEqual(["time"]);
    });

    it("extracts multiple arguments", () => {
      expect(
        icuProcessor.extractTokens(
          "Hello {name}, you have {count, plural, one {# item} other {# items}}",
        ),
      ).toEqual(["name", "count"]);
    });

    it("extracts nested select with nested arguments", () => {
      expect(
        icuProcessor.extractTokens("{gender, select, male {{name} is here} other {They are here}}"),
      ).toEqual(["gender", "name"]);
    });

    it("returns empty for plain string", () => {
      expect(icuProcessor.extractTokens("Hello world")).toEqual([]);
    });
  });

  describe("validate", () => {
    it("returns valid when tokens match", () => {
      const result = icuProcessor.validate(
        "{count, plural, one {# item} other {# items}}",
        "{count, plural, one {# article} other {# articles}}",
      );
      expect(result).toEqual({ valid: true });
    });

    it("returns error when source tokens are missing in translation", () => {
      const result = icuProcessor.validate(
        "{count, plural, one {# item} other {# items}}",
        "Tienes artículos",
      );
      expect(result.valid).toBe(false);
      expect(result.errors?.some((e) => e.includes("Missing tokens: count"))).toBe(true);
    });
  });

  describe("getSyntaxHint", () => {
    it("returns a hint string", () => {
      expect(typeof icuProcessor.getSyntaxHint()).toBe("string");
      expect(icuProcessor.getSyntaxHint().length).toBeGreaterThan(0);
    });
  });
});
