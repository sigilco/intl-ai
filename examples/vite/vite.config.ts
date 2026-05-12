import { defineConfig } from "vite";
import intlAi from "@intl-ai/unplugin/vite";

export default defineConfig({
  plugins: [intlAi()],
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
