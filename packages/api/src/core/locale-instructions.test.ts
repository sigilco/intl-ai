import { describe, expect, it } from "vitest";
import { findUnmatchedInstructionKeys, resolveLocaleInstruction } from "./locale-instructions";

describe("resolveLocaleInstruction", () => {
  it("returns undefined when instructions are absent", () => {
    expect(resolveLocaleInstruction(undefined, "en-GB")).toBeUndefined();
  });

  it("resolves an exact locale match", () => {
    const instructions = { "en-GB": "Use British spelling.", en: "Use American spelling." };
    expect(resolveLocaleInstruction(instructions, "en-GB")).toBe("Use British spelling.");
  });

  it("falls back to the language subtag", () => {
    const instructions = { en: "Use American spelling." };
    expect(resolveLocaleInstruction(instructions, "en-US")).toBe("Use American spelling.");
  });

  it("falls back to the wildcard key", () => {
    const instructions = { "*": "Be formal." };
    expect(resolveLocaleInstruction(instructions, "fr-FR")).toBe("Be formal.");
  });

  it("prefers exact match over subtag over wildcard", () => {
    const instructions = { "en-GB": "exact", en: "subtag", "*": "wildcard" };
    expect(resolveLocaleInstruction(instructions, "en-GB")).toBe("exact");
    expect(resolveLocaleInstruction(instructions, "en-US")).toBe("subtag");
    expect(resolveLocaleInstruction(instructions, "fr-FR")).toBe("wildcard");
  });

  it("returns undefined on a total miss", () => {
    const instructions = { en: "Use American spelling." };
    expect(resolveLocaleInstruction(instructions, "fr-FR")).toBeUndefined();
  });
});

describe("findUnmatchedInstructionKeys", () => {
  it("returns no keys when instructions are absent", () => {
    expect(findUnmatchedInstructionKeys(undefined, ["en-GB"])).toEqual([]);
  });

  it("flags a key that matches no configured locale or subtag", () => {
    const instructions = { fr: "Formal." };
    expect(findUnmatchedInstructionKeys(instructions, ["en-GB", "en-US"])).toEqual(["fr"]);
  });

  it("does not flag an exact locale match", () => {
    const instructions = { "en-GB": "British." };
    expect(findUnmatchedInstructionKeys(instructions, ["en-GB", "en-US"])).toEqual([]);
  });

  it("does not flag a subtag match", () => {
    const instructions = { en: "American." };
    expect(findUnmatchedInstructionKeys(instructions, ["en-GB", "en-US"])).toEqual([]);
  });

  it("never flags the wildcard key", () => {
    const instructions = { "*": "Be formal." };
    expect(findUnmatchedInstructionKeys(instructions, ["en-GB"])).toEqual([]);
  });
});
