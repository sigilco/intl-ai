import { describe, test, expect, beforeEach, afterEach } from "vitest";
import { checkAction } from "./check";
import {
  createTempDir,
  cleanupTempDir,
  sampleLocaleEn,
  sampleLocaleFr,
  createLocaleFile,
} from "../../../core/src/testing/fixtures";
import { writeFileSync, mkdirSync } from "fs";
import { join } from "path";

describe("check command", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = createTempDir("check-test");
  });

  afterEach(() => {
    cleanupTempDir(tempDir);
  });

  test("should report missing translations", async () => {
    const localeDir = join(tempDir, "locales");
    mkdirSync(localeDir, { recursive: true });

    const configPath = join(tempDir, "intl-ai.config.ts");
    writeFileSync(
      configPath,
      `
export default {
  defaultLocale: "en",
  locales: ["en", "fr"],
  localeDir: "${localeDir}",
  model: { id: "test-model" },
};
`,
    );

    createLocaleFile(localeDir, "en", sampleLocaleEn);
    createLocaleFile(localeDir, "fr", sampleLocaleFr);

    const originalCwd = process.cwd();
    process.chdir(tempDir);

    try {
      let exitCode = 0;
      const originalExit = process.exit;
      process.exit = ((code: number) => {
        exitCode = code;
      }) as never;

      await checkAction({ locale: "fr" });

      process.exit = originalExit;
      expect(exitCode).toBe(10);
    } finally {
      process.chdir(originalCwd);
    }
  });

  test("should exit 0 when all translations complete", async () => {
    const localeDir = join(tempDir, "locales");
    mkdirSync(localeDir, { recursive: true });

    const configPath = join(tempDir, "intl-ai.config.ts");
    writeFileSync(
      configPath,
      `
export default {
  defaultLocale: "en",
  locales: ["en", "fr"],
  localeDir: "${localeDir}",
  model: { id: "test-model" },
};
`,
    );

    createLocaleFile(localeDir, "en", sampleLocaleEn);
    createLocaleFile(localeDir, "fr", sampleLocaleEn);

    const originalCwd = process.cwd();
    process.chdir(tempDir);

    try {
      let exitCode = 0;
      const originalExit = process.exit;
      process.exit = ((code: number) => {
        exitCode = code;
      }) as never;

      await checkAction({ locale: "fr" });

      process.exit = originalExit;
      expect(exitCode).toBe(0);
    } finally {
      process.chdir(originalCwd);
    }
  });

  test("should support --locale option", async () => {
    const localeDir = join(tempDir, "locales");
    mkdirSync(localeDir, { recursive: true });

    const configPath = join(tempDir, "intl-ai.config.ts");
    writeFileSync(
      configPath,
      `
export default {
  defaultLocale: "en",
  locales: ["en", "fr", "es"],
  localeDir: "${localeDir}",
  model: { id: "test-model" },
};
`,
    );

    createLocaleFile(localeDir, "en", sampleLocaleEn);
    createLocaleFile(localeDir, "fr", sampleLocaleFr);
    createLocaleFile(localeDir, "es", sampleLocaleEn);

    const originalCwd = process.cwd();
    process.chdir(tempDir);

    try {
      let exitCode = 0;
      const originalExit = process.exit;
      process.exit = ((code: number) => {
        exitCode = code;
      }) as never;

      await checkAction({ locale: "fr" });

      process.exit = originalExit;
      expect(exitCode).toBe(10);
    } finally {
      process.chdir(originalCwd);
    }
  });
});
