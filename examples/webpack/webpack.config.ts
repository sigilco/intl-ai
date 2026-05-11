import type { Configuration } from "webpack";
import IntlAiPlugin from "@intl-ai/unplugin/webpack";

const config: Configuration = {
  entry: "./src/index.ts",
  mode: "development",
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: [".tsx", ".ts", ".js"],
  },
  plugins: [IntlAiPlugin()],
};

export default config;
