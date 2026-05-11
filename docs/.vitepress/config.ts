import { defineConfig } from "vitepress";
import {
  groupIconMdPlugin,
  groupIconVitePlugin,
} from "vitepress-plugin-group-icons";

export default defineConfig({
  title: "unplugin-intl-ai",
  description: "AI-powered i18n translation for any bundler",
  base: "/unplugin-intl-ai/",
  markdown: {
    config(md) {
      md.use(groupIconMdPlugin);
    },
  },
  vite: {
    plugins: [groupIconVitePlugin()],
  },
  themeConfig: {
    nav: [
      { text: "Guide", link: "/guide/getting-started" },
      { text: "GitHub", link: "https://github.com/joaquin.terrasamoya/intl-ai" },
    ],
    sidebar: {
      "/guide/": [
        {
          text: "Getting Started",
          collapsed: false,
          items: [
            { text: "Getting Started", link: "/guide/getting-started" },
            { text: "AI Model Setup", link: "/guide/ai-model" },
          ],
        },
        {
          text: "Framework Integration",
          collapsed: false,
          items: [
            { text: "Next.js", link: "/guide/next-js" },
            { text: "Vue (vue-i18n)", link: "/guide/vue-i18n" },
            { text: "i18next", link: "/guide/i18next" },
          ],
        },
        {
          text: "Project",
          collapsed: false,
          items: [
            { text: "SDK Reference", link: "/guide/configuration" },
            { text: "Contributing", link: "/guide/contributing" },
          ],
        },
      ],
    },
    editLink: {
      pattern:
        "https://github.com/joaquin.terrasamoya/intl-ai/edit/main/docs/:path",
    },
  },
});
