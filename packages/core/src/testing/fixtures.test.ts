// packages/core/src/testing/fixtures.test.ts

import { describe, test, expect, beforeEach, afterEach } from "vitest";
import {
  createTempDir,
  cleanupTempDir,
  sampleLocaleEn,
  assertTranslationEquals,
  assertHasKey,
} from "./fixtures";

describe("Test Utilities", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = createTempDir();
  });

  afterEach(() => {
    cleanupTempDir(tempDir);
  });

  test("createTempDir creates a directory", () => {
    expect(tempDir).toBeDefined();
    expect(typeof tempDir).toBe("string");
  });

  test("sampleLocaleEn has expected keys", () => {
    assertHasKey(sampleLocaleEn, "greeting");
    assertHasKey(sampleLocaleEn, "farewell");
    expect(sampleLocaleEn.greeting).toBe("Hello");
  });

  test("assertTranslationEquals works correctly", () => {
    assertTranslationEquals("Hello", "Hello");
    expect(() => assertTranslationEquals("Hello", "Bonjour")).toThrow();
  });
});
