import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["packages/*/src/**/*.test.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov", "json-summary"],
      thresholds: { lines: 80, branches: 80 },
      include: ["packages/*/src/**"],
      exclude: ["**/*.test.ts", "**/testing/**", "**/dist/**"],
    },
  },
});
