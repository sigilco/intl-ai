import { defineConfig } from "vitepress";
import { groupIconMdPlugin, groupIconVitePlugin } from "vitepress-plugin-group-icons";
import llmstxt from "vitepress-plugin-llms";
import { SITE_URL, URLs } from "./urls";

// English nav + sidebar are the source of truth; non-default locales derive
// their own copies via prefixLinks (translated pages live under /<locale>/).
const navItems = [
  { text: "Guide", link: "/guide/getting-started" },
  { text: "Sponsor", link: URLs.sponsor },
  { text: "GitHub", link: URLs.repo },
  {
    text: "LLMs",
    items: [
      { text: "llms.txt", link: "/llms.txt" },
      { text: "llms-full.txt", link: "/llms-full.txt" },
    ],
  },
];

const sidebarGuide = [
  {
    text: "Getting started",
    collapsed: false,
    items: [
      { text: "Getting started", link: "/guide/getting-started" },
      { text: "AI model setup", link: "/guide/ai-model" },
      { text: "Configuration", link: "/guide/configuration" },
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
      { text: "i18n libraries", link: "/guide/i18n-libraries/" },
      { text: "Vue (vue-i18n)", link: "/guide/i18n-libraries/vue-i18n" },
      { text: "i18next", link: "/guide/i18n-libraries/i18next" },
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
];

// Prefix only internal /guide links; external URLs and the English-only /llms.txt stay as-is.
function prefixLinks<T extends { link?: string; items?: T[] }>(items: T[], prefix: string): T[] {
  return items.map((item) => {
    const next = { ...item } as T;
    if (item.link !== undefined) {
      (next as { link: string }).link = item.link.startsWith("/guide")
        ? prefix + item.link
        : item.link;
    }
    if (item.items !== undefined) {
      (next as { items: T[] }).items = prefixLinks(item.items, prefix);
    }
    return next;
  });
}

export default defineConfig({
  title: "intl-ai",
  description: "AI-powered i18n translation. Zero runtime dependencies.",
  cleanUrls: true,
  sitemap: {
    hostname: SITE_URL,
  },
  head: [
    ["link", { rel: "icon", type: "image/svg+xml", href: "/logo.svg" }],
    ["link", { rel: "canonical", href: SITE_URL }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:site_name", content: "intl-ai" }],
    ["meta", { property: "og:title", content: "intl-ai" }],
    [
      "meta",
      {
        property: "og:description",
        content: "AI-powered i18n translation. Zero runtime dependencies.",
      },
    ],
    ["meta", { property: "og:image", content: `${SITE_URL}/logo-512x512.png` }],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    ["meta", { name: "twitter:title", content: "intl-ai" }],
    [
      "meta",
      {
        name: "twitter:description",
        content: "AI-powered i18n translation. Zero runtime dependencies.",
      },
    ],
    ["meta", { name: "twitter:image", content: `${SITE_URL}/logo-512x512.png` }],
    [
      "script",
      {
        type: "application/ld+json",
        innerHTML: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "SoftwareApplication",
          name: "intl-ai",
          url: SITE_URL,
          description:
            "AI-powered i18n translation plugin for any bundler. Zero runtime dependencies.",
          applicationCategory: "DeveloperApplication",
          offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
        }),
      },
    ],
    [
      "script",
      {
        type: "application/ld+json",
        innerHTML: JSON.stringify({
          "@context": "https://schema.org",
          "@type": "TechArticle",
          url: SITE_URL,
          headline: "intl-ai documentation",
          description: "AI-powered i18n translation. Zero runtime dependencies.",
        }),
      },
    ],
  ],
  base: "/",
  locales: {
    root: { label: "English", lang: "en" },
  },
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
    nav: navItems,
    sidebar: { "/guide/": sidebarGuide },
    editLink: {
      pattern: "https://github.com/sigilco/intl-ai/edit/main/docs/:path",
    },
    search: {
      provider: "local",
    },
  },
});
