import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["index.ts"],
  outDir: "dist",
  format: "esm",
  dts: true,
  clean: true,
  sourcemap: false,
  bundle: false,
});
