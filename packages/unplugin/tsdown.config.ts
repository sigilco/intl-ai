import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["src/!(*.test).ts"],
  format: ["esm", "cjs"],
  dts: true,
  sourcemap: true,
  clean: true,
  treeshake: true,
  external: [
    "vite",
    "rollup",
    "webpack",
    "esbuild",
    "@rspack/core",
    "rolldown",
    "@farmfe/core",
    "bun",
    "intl-ai",
  ],
});
