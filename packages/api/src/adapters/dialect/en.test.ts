import { describe, expect, it } from "vitest";
import { detectDialect, tokenSpans } from "./en";

describe("detectDialect", () => {
  it("flags a British spelling when the expected variant is american", async () => {
    const hits = await detectDialect("I love this colour scheme.", "american");
    expect(hits).toHaveLength(1);
    expect(hits[0]).toMatchObject({ term: "colour", suggestion: "color" });
  });

  it("flags an American spelling when the expected variant is british", async () => {
    const hits = await detectDialect("I love this color scheme.", "british");
    expect(hits).toHaveLength(1);
    expect(hits[0]).toMatchObject({ term: "color", suggestion: "colour" });
  });

  it("preserves capitalization in the suggestion", async () => {
    const hits = await detectDialect("Colour", "american");
    expect(hits[0]?.suggestion).toBe("Color");
  });

  it("preserves all-caps in the suggestion", async () => {
    const hits = await detectDialect("COLOUR", "american");
    expect(hits[0]?.suggestion).toBe("COLOR");
  });

  it("finds no hits inside an ICU placeholder when excluded", async () => {
    const text = "Pick your {colour} scheme.";
    const spans = tokenSpans(text, ["{colour}"]);
    const hits = await detectDialect(text, "american", spans);
    expect(hits).toHaveLength(0);
  });

  it("finds no hits inside a plural/select block when excluded", async () => {
    const text = "{count, plural, one {# colour} other {# colours}}";
    const spans = tokenSpans(text, [text]);
    const hits = await detectDialect(text, "american", spans);
    expect(hits).toHaveLength(0);
  });

  it("returns no hits on already-American text", async () => {
    const hits = await detectDialect("I love this color and this flavor.", "american");
    expect(hits).toHaveLength(0);
  });
});
