import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    // Real HTTP/subprocess calls against live services; local-only, run via `pnpm test:integration`.
    exclude: ["**/*.integration.test.ts", "**/node_modules/**"],
    environment: "node",
  },
});
