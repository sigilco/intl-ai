import { describe, test, expect, beforeEach, afterEach } from "vitest";
import { checkCommand } from "./check";
import { mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "pathe";

const sampleLocaleEn = {
  greeting: "Hello",
  farewell: "Goodbye",
  nested: {
    message: "Welcome",
  },
};

const sampleLocaleFr = {
  greeting: "Bonjour",
  // farewell missing intentionally
  nested: {
    message: "Bienvenue",
  },
};

describe("check command", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = join(tmpdir(), `check-test-${Date.now()}-${Math.random().toString(36).slice(2)}`);
    mkdirSync(tempDir, { recursive: true });
  });

  afterEach(() => {
    rmSync(tempDir, { recursive: true, force: true });
  });

  function createConfig(localeDir: string) {
    writeFileSync(
      join(tempDir, "intl-ai.config.json"),
      JSON.stringify({
        defaultLocale: "en",
        locales: ["en", "fr"],
        localeDir,
        model: "gpt-4o-mini",
        apiKey: "test-key",
      }),
    );
  }

  function createLocales(localeDir: string, fr: Record<string, unknown>) {
    mkdirSync(localeDir, { recursive: true });
    writeFileSync(join(localeDir, "en.json"), JSON.stringify(sampleLocaleEn, null, 2));
    writeFileSync(join(localeDir, "fr.json"), JSON.stringify(fr, null, 2));
  }

  test("module exports check command", async () => {
    expect(checkCommand).toBeDefined();
    const meta = await Promise.resolve(checkCommand.meta as any);
    expect(meta?.name).toBe("check");
  });

  test("should report missing translations", async () => {
    const localeDir = join(tempDir, "locales");
    createConfig(localeDir);
    createLocales(localeDir, sampleLocaleFr);

    const originalCwd = process.cwd();
    process.chdir(tempDir);

    try {
      let exitCode = 0;
      const originalExit = process.exit;
      process.exit = ((code: number) => {
        exitCode = code;
      }) as never;

      await checkCommand.run?.({ args: { config: "intl-ai.config.json" } } as any);

      process.exit = originalExit;
      expect(exitCode).toBe(10);
    } finally {
      process.chdir(originalCwd);
    }
  });

  test("should exit 0 when all translations complete", async () => {
    const localeDir = join(tempDir, "locales");
    createConfig(localeDir);
    createLocales(localeDir, sampleLocaleEn);

    const originalCwd = process.cwd();
    process.chdir(tempDir);

    try {
      let exitCode = 0;
      const originalExit = process.exit;
      process.exit = ((code: number) => {
        exitCode = code;
      }) as never;

      await checkCommand.run?.({ args: { config: "intl-ai.config.json" } } as any);

      process.exit = originalExit;
      expect(exitCode).toBe(0);
    } finally {
      process.chdir(originalCwd);
    }
  });

  test("should support --locale option", async () => {
    const localeDir = join(tempDir, "locales");
    createConfig(localeDir);
    createLocales(localeDir, sampleLocaleFr);
    mkdirSync(localeDir, { recursive: true });
    writeFileSync(join(localeDir, "es.json"), JSON.stringify(sampleLocaleEn, null, 2));

    const originalCwd = process.cwd();
    process.chdir(tempDir);

    try {
      let exitCode = 0;
      const originalExit = process.exit;
      process.exit = ((code: number) => {
        exitCode = code;
      }) as never;

      await checkCommand.run?.({ args: { config: "intl-ai.config.json", locale: "fr" } } as any);

      process.exit = originalExit;
      expect(exitCode).toBe(10);
    } finally {
      process.chdir(originalCwd);
    }
  });
});
