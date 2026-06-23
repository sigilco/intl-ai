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
            { text: "i18n libraries", link: "/guide/i18n-libraries" },
          ],
        },
        {
          text: "Build systems",
          collapsed: true,
          items: [
            { text: "Build systems", link: "/guide/build-systems/" },
            { text: "Vite", link: "/guide/build-systems/vite" },
            { text: "Webpack", link: "/guide/build-systems/webpack" },
            { text: "Rollup", link: "/guide/build-systems/rollup" },
            { text: "esbuild", link: "/guide/build-systems/esbuild" },
            { text: "Rspack", link: "/guide/build-systems/rspack" },
            { text: "Rolldown", link: "/guide/build-systems/rolldown" },
            { text: "Farm", link: "/guide/build-systems/farm" },
            { text: "Next.js", link: "/guide/build-systems/next-js" },
          ],
        },
        {
          text: "i18n libraries",
          collapsed: true,
          items: [
            { text: "i18n libraries", link: "/guide/i18n-libraries" },
            { text: "Vue (vue-i18n)", link: "/guide/vue-i18n" },
            { text: "i18next", link: "/guide/i18next" },
          ],
        },
        {
          text: "Mobile & desktop",
          collapsed: true,
          items: [
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
            { text: "Contributing", link: "/guide/contributing" },
            { text: "Internals", link: "/guide/internals" },
          ],
        },
      ],
    },
    editLink: {
      pattern: "https://github.com/sigilco/intl-ai/edit/main/docs/:path",
    },
  },
});
