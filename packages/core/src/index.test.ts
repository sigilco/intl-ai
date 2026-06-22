import { describe, it, expect } from "vitest";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "pathe";
import { runFill, loadConfig, type IntlAiConfig } from "./index";

describe("@intl-ai/core re-exports", () => {
  it("re-exports the lean public API surface", () => {
    expect(typeof runFill).toBe("function");
  });

  it("IntlAiConfig re-export is structurally compatible with the api shape", () => {
    const cfg: IntlAiConfig = {
      defaultLocale: "en",
      locales: ["en", "es"],
      localeDir: "./locales",
      model: { modelId: "gpt-4o-mini" },
    };
    expect(cfg.defaultLocale).toBe("en");
  });
});

describe("loadConfig (legacy TS config support)", () => {
  it("loads a TypeScript intl-ai.config.ts via jiti", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-core-"));
    try {
      await writeFile(
        join(dir, "intl-ai.config.ts"),
        `export default {
          defaultLocale: "en",
          locales: ["en", "fr"],
          localeDir: "./locales",
          model: { modelId: "gpt-4o-mini", config: { apiKey: "x", baseURL: "https://x" } },
        };`,
        "utf-8",
      );
      const cfg = await loadConfig(dir);
      expect(cfg.defaultLocale).toBe("en");
      expect(cfg.locales).toEqual(["en", "fr"]);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("throws a clear error when no config is found", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-core-"));
    try {
      await expect(loadConfig(dir)).rejects.toThrow(/No intl-ai config found/);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
