import { describe, test, expect } from "vitest";
import { unplugin } from "./index";

describe("@intl-ai/unplugin", () => {
  test("can be imported and initialized", () => {
    expect(unplugin).toBeDefined();
    expect(typeof unplugin.raw).toBe("function");
    expect(typeof unplugin.vite).toBe("function");
    expect(typeof unplugin.webpack).toBe("function");
    expect(typeof unplugin.rollup).toBe("function");
    expect(typeof unplugin.esbuild).toBe("function");
    expect(typeof unplugin.rspack).toBe("function");
  });

  test("creates plugin with correct name", () => {
    const plugin = unplugin.raw({}, { root: "" } as any);
    expect((plugin as any).name).toBe("@intl-ai/unplugin");
    expect((plugin as any).buildStart).toBeDefined();
  });

  test("accepts options", () => {
    const plugin = unplugin.raw({ debug: true }, { root: "" } as any);
    expect((plugin as any).name).toBe("@intl-ai/unplugin");
  });
});
