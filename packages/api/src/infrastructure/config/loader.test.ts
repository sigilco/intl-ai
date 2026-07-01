import { describe, it, expect } from "vitest";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { loadConfigFromPath } from "./loader";

const baseConfig = {
  defaultLocale: "en",
  locales: ["en", "es"],
  localeDir: "locales",
  provider: "openai",
  model: "gpt-4o-mini",
  apiKey: "k",
};

async function writeConfig(dir: string, json: object): Promise<string> {
  const path = join(dir, "intl-ai.config.json");
  await writeFile(path, JSON.stringify(json), "utf8");
  return path;
}

describe("config loader", () => {
  it("resolves a relative localeDir against the config file directory", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-"));
    const path = await writeConfig(dir, baseConfig);
    const config = await loadConfigFromPath(path);
    expect(config.localeDir).toBe(join(dir, "locales"));
  });

  it("leaves an absolute localeDir unchanged", async () => {
    const dir = await mkdtemp(join(tmpdir(), "intl-ai-"));
    const abs = join(dir, "elsewhere");
    const path = await writeConfig(dir, { ...baseConfig, localeDir: abs });
    const config = await loadConfigFromPath(path);
    expect(config.localeDir).toBe(abs);
  });
});
