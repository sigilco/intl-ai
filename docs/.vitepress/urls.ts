// Centralized URLs for the intl-ai docs site.
// Keep this as the single source of truth. Reference these constants from
// `.vitepress/config.ts` (e.g. `editLink.pattern`, `nav[].link`) and from the
// `<DocsUrl>` global component when authors need to embed a link in prose.

export const SITE_URL = "https://intl-ai.pages.dev";

export const URLs = {
  site: SITE_URL,
  repo: "https://github.com/sigilco/intl-ai",
  issues: "https://github.com/sigilco/intl-ai/issues",
  npmOrg: "https://www.npmjs.com/org/intl-ai",
  npm: {
    api: "https://www.npmjs.com/package/@intl-ai/api",
    cli: "https://www.npmjs.com/package/@intl-ai/cli",
    unplugin: "https://www.npmjs.com/package/@intl-ai/unplugin",
    next: "https://www.npmjs.com/package/@intl-ai/next",
  },
  schema: "https://intl-ai.pages.dev/schema/v1.json",
  installScript: "https://intl-ai.pages.dev/install.sh",
  homebrew: "brew install sigilco/tap/intl-ai",
  mise: 'mise use npm:intl-ai@latest',
} as const;

export type UrlKey = keyof typeof URLs;
