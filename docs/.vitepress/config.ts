import { defineConfig } from "vitepress";
import { groupIconMdPlugin, groupIconVitePlugin } from "vitepress-plugin-group-icons";
import llmstxt from "vitepress-plugin-llms";

export default defineConfig({
  title: "intl-ai",
  description: "AI-powered i18n translation. Zero runtime dependencies.",
  base: "/",
  markdown: {
    config(md) {
      md.use(groupIconMdPlugin);
    },
  },
  vite: {
    plugins: [groupIconVitePlugin(), llmstxt()],
  },
  themeConfig: {
    logo: { src: "/logo.svg", alt: "intl-ai" },
    nav: [
      { text: "Guide", link: "/guide/getting-started" },
      { text: "GitHub", link: "https://github.com/sigilco/intl-ai" },
    ],
    sidebar: {
      "/guide/": [
        {
          text: "Getting started",
          collapsed: false,
          items: [
            { text: "Getting started", link: "/guide/getting-started" },
            { text: "Installation", link: "/guide/installation" },
            { text: "AI model setup", link: "/guide/ai-model" },
            { text: "Configuration", link: "/guide/configuration" },
          ],
        },
        {
          text: "Framework integration",
          collapsed: false,
          items: [
            { text: "Next.js", link: "/guide/next-js" },
            { text: "Vue (vue-i18n)", link: "/guide/vue-i18n" },
            { text: "i18next", link: "/guide/i18next" },
            { text: "Expo", link: "/guide/mobile/expo" },
            { text: "Flutter", link: "/guide/mobile/flutter" },
            { text: "SwiftUI", link: "/guide/mobile/swiftui" },
            { text: "Android Jetpack", link: "/guide/mobile/jetpack" },
            { text: ".NET", link: "/guide/desktop/dotnet" },
          ],
        },
        {
          text: "Reference",
          collapsed: false,
          items: [
            { text: "API reference", link: "/guide/api" },
            { text: "Migration", link: "/guide/migration" },
            { text: "Contributing", link: "/guide/contributing" },
          ],
        },
      ],
    },
    editLink: {
      pattern: "https://github.com/sigilco/intl-ai/edit/main/docs/:path",
    },
  },
});
