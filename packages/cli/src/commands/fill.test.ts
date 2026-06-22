import { describe, test, expect } from "vitest";

describe("fill command", () => {
  test("module can be imported without error", async () => {
    const { fillCommand } = await import("./fill");

    expect(fillCommand).toBeDefined();

    const meta = await Promise.resolve(fillCommand.meta as any);
    expect(meta?.name).toBe("fill");
    expect(meta?.description).toBe("Fill missing translations using AI");
  });

  test("command args are configured correctly", async () => {
    const { fillCommand } = await import("./fill");
    const args = await Promise.resolve(fillCommand.args as any);

    expect(args).toBeDefined();
    expect(args?.config?.type).toBe("string");
    expect(args?.config?.default).toBe("intl-ai.config.json");
    expect(args?.locale?.type).toBe("string");
    expect(args?.force?.type).toBe("boolean");
    expect(args?.force?.default).toBe(false);
    expect(args?.silent?.type).toBe("boolean");
    expect(args?.["dry-run"]?.type).toBe("boolean");
  });
});
