import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["src/!(*.test).ts"],
  format: "esm",
  dts: true,
  sourcemap: true,
  clean: true,
  external: [
    "vite",
    "rollup",
    "webpack",
    "esbuild",
    "@rspack/core",
    "rolldown",
    "@farmfe/core",
    "next",
    "@intl-ai/next",
  ],
});
