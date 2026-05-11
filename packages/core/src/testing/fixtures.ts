// packages/core/src/testing/fixtures.ts

import { writeFileSync, mkdirSync, rmSync, existsSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

// Temp directory management
export function createTempDir(prefix = "intl-ai-test-"): string {
  const dir = join(tmpdir(), `${prefix}${Date.now()}`);
  mkdirSync(dir, { recursive: true });
  return dir;
}

export function cleanupTempDir(dir: string): void {
  if (existsSync(dir)) {
    rmSync(dir, { recursive: true, force: true });
  }
}

// Sample locale fixtures
export const sampleLocaleEn = {
  greeting: "Hello",
  farewell: "Goodbye",
  nested: {
    message: "Welcome",
  },
};

export const sampleLocaleFr = {
  greeting: "Bonjour",
  farewell: "Au revoir",
  // nested.message is missing - for testing diff
};

export const sampleLocaleYaml = `
greeting: Hello
farewell: Goodbye
nested:
  message: Welcome
`;

// Locale file creation helper
export function createLocaleFile(
  dir: string,
  locale: string,
  content: Record<string, unknown>,
  format: "json" | "yaml" = "json",
): string {
  const filename = format === "json" ? `${locale}.json` : `${locale}.yaml`;
  const filepath = join(dir, filename);

  if (format === "json") {
    writeFileSync(filepath, JSON.stringify(content, null, 2));
  } else {
    writeFileSync(filepath, content as unknown as string);
  }

  return filepath;
}

// Assertion helpers
export function assertTranslationEquals(actual: string, expected: string, message?: string): void {
  if (actual !== expected) {
    throw new Error(message || `Expected "${expected}" but got "${actual}"`);
  }
}

export function assertHasKey(obj: Record<string, unknown>, key: string, message?: string): void {
  if (!(key in obj)) {
    throw new Error(message || `Expected key "${key}" to exist`);
  }
}
