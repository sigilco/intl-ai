import { describe, test, expect } from "vitest";

describe("fill command", () => {
  test("module can be imported without error", async () => {
    const { fillCommand } = await import("./fill");

    expect(fillCommand).toBeDefined();
    expect(fillCommand.options.name).toBe("fill");
  });

  test("command flags are configured correctly", async () => {
    const { fillCommand } = await import("./fill");
    const { flags } = fillCommand.options;

    expect(flags).toBeDefined();
    expect(flags?.config?.type).toBe(String);
    expect(flags?.config?.default).toBe("intl-ai.config.json");
    expect(flags?.locale?.type).toBe(String);
    expect(flags?.force?.type).toBe(Boolean);
    expect(flags?.force?.default).toBe(false);
    expect(flags?.silent?.type).toBe(Boolean);
    expect(flags?.dryRun?.type).toBe(Boolean);
  });
});
