// Cloudflare Pages builds `pnpm docs:build` and publishes docs/.vitepress/dist,
// so `out` deliberately keeps that directory. Rename to `site/` (and update the
// Pages project's output directory) once the dashboard config can change.
export default {
  title: "intl-ai",
  url: "https://intl-ai.pages.dev",
  src: "docs",
  out: "docs/.vitepress/dist",
  base: "/",
  engine: "js",
  logo: {
    light: "assets/logo.svg",
    dark: "assets/logo.svg",
    href: "/",
    alt: "intl-ai",
    height: "28px",
  },
  favicon: "assets/logo.svg",
  layout: {
    spa: true,
    header: {
      enabled: true,
    },
    sidebar: {
      collapsible: true,
      defaultCollapsed: false,
    },
    optionsMenu: {
      position: "sidebar-top",
      components: {
        search: true,
        themeSwitch: true,
      },
    },
    footer: {
      style: "minimal",
      content: "Apache-2.0 license.",
      branding: false,
    },
  },
  theme: {
    name: "default",
    appearance: "system",
    codeHighlight: true,
  },
  minify: true,
  autoTitleFromH1: true,
  copyCode: true,
  pageNavigation: true,
  navigation: [
    { title: "Home", path: "/", icon: "house" },
    {
      title: "Getting started",
      icon: "rocket",
      collapsible: true,
      children: [
        { title: "Getting started", path: "/guide/getting-started/" },
        { title: "AI model setup", path: "/guide/ai-model/" },
        { title: "Configuration", path: "/guide/configuration/" },
        { title: "Providers", path: "/guide/providers/" },
        { title: "Observability", path: "/guide/observability/" },
      ],
    },
    {
      title: "Build systems",
      icon: "hammer",
      collapsible: true,
      children: [
        { title: "Overview", path: "/guide/build-systems/" },
        { title: "Vite", path: "/guide/build-systems/vite/" },
        { title: "Webpack", path: "/guide/build-systems/webpack/" },
        { title: "Rollup", path: "/guide/build-systems/rollup/" },
        { title: "esbuild", path: "/guide/build-systems/esbuild/" },
        { title: "Rspack", path: "/guide/build-systems/rspack/" },
        { title: "Rolldown", path: "/guide/build-systems/rolldown/" },
        { title: "Farm", path: "/guide/build-systems/farm/" },
        { title: "Next.js", path: "/guide/build-systems/next-js/" },
      ],
    },
    {
      title: "i18n libraries",
      icon: "languages",
      collapsible: true,
      children: [
        { title: "Overview", path: "/guide/i18n-libraries/" },
        { title: "Vue (vue-i18n)", path: "/guide/i18n-libraries/vue-i18n/" },
        { title: "i18next", path: "/guide/i18n-libraries/i18next/" },
      ],
    },
    {
      title: "Mobile and desktop",
      icon: "smartphone",
      collapsible: true,
      children: [
        { title: "Expo", path: "/guide/mobile/expo/" },
        { title: "Flutter", path: "/guide/mobile/flutter/" },
        { title: "SwiftUI", path: "/guide/mobile/swiftui/" },
        { title: "Android Jetpack", path: "/guide/mobile/jetpack/" },
        { title: ".NET", path: "/guide/desktop/dotnet/" },
      ],
    },
    {
      title: "Ecosystem",
      icon: "puzzle",
      collapsible: true,
      children: [{ title: "Community integrations", path: "/guide/community-plugins/" }],
    },
    {
      title: "Reference",
      icon: "book-open",
      collapsible: true,
      children: [
        { title: "API reference", path: "/guide/api/" },
        { title: "Internals", path: "/guide/internals/" },
        { title: "Contributing", path: "/guide/contributing/" },
      ],
    },
    {
      title: "LLMs",
      icon: "file-text",
      collapsible: true,
      children: [
        { title: "llms.txt", path: "/llms.txt", external: true },
        { title: "llms-full.txt", path: "/llms-full.txt", external: true },
      ],
    },
    {
      title: "Sponsor",
      path: "https://buy.polar.sh/polar_cl_Mv1gdlG7bw3I70EC9IHtfeSHJj4PEKvA7JAUz23CFhj",
      icon: "heart",
      external: true,
    },
    {
      title: "GitHub",
      path: "https://github.com/sigilco/intl-ai",
      icon: "github",
      external: true,
    },
  ],
  plugins: {
    ai: {
      assistant: false,
    },
    git: {
      commitHistory: true,
      maxCommits: 5,
      repository: "https://github.com/sigilco/intl-ai",
      branch: "develop",
      editLink: true,
    },
    llms: {
      enabled: true,
      fullContext: true,
    },
    seo: {
      defaultDescription:
        "AI-powered i18n translation at build time. One binary, any bundler, any model. Zero runtime dependencies.",
    },
  },
};
