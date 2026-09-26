import { defineConfig } from "vite";
import intlAi from "@intl-ai/unplugin/vite";

const intlAiPlugin = process.env.NODE_ENV === "production" ? null : intlAi();

export default defineConfig({
  plugins: [intlAiPlugin],
  build: {
    commonjsOptions: {
      ignoreDynamicRequires: true,
    },
    rollupOptions: {
      external: [
        "@intl-ai/core",
        "@intl-ai/unplugin",
        "@intl-ai/unplugin/vite",
        "unplugin",
        "jiti",
      ],
    },
  },
});
