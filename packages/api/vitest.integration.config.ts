import { defineConfig } from "vitest/config";
import { config as loadDotenv } from "dotenv";
import { fileURLToPath } from "node:url";

const { parsed } = loadDotenv({ path: fileURLToPath(new URL("../../.env", import.meta.url)) });

export default defineConfig({
  test: {
    include: ["src/**/*.integration.test.ts"],
    environment: "node",
    env: parsed,
  },
});
