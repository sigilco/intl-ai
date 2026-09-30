---
title: Contributing
description: Contribute to intl-ai documentation. Edit any doc page on GitHub and open a pull request.
---

# Contributing to the documentation

Thank you for your interest in improving the intl-ai documentation. This guide covers everything you need to know about contributing to the docs.

For general contribution guidelines, branch workflows, commit conventions, and the full development process, see our [main Contributing Guide](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Documentation setup

The docs are built with [docmd](https://docmd.io/), a Markdown-first static site generator.

### Prerequisites

- Node.js 22+ and pnpm 11+
- Basic familiarity with Markdown
- A text editor (VS Code recommended)

### Installation

```bash
pnpm install
```

## Running docs locally

Start the development server to preview your changes in real-time:

```bash
pnpm docs:dev
```

This starts a local dev server (typically at `http://localhost:3000`) with live reload for instant preview updates.

## Building for production

```bash
pnpm docs:build
```

This generates optimized static files in the `docs/.vitepress/dist/` directory and copies `docs/public/` assets to the site root.

## Previewing the production build

After building, serve the output directory with any static file server to verify the production output.

## Documentation writing guidelines

### File structure

```
docmd.config.js     # site config, navigation, plugins
assets/             # images served at /assets/
docs/
  guide/
    getting-started.md
    ai-model.md
    configuration.md
    api.md
    contributing.md
    build-systems/
    i18n-libraries/
    mobile/
    desktop/
  public/           # files copied to the site root on build
    install.sh
    schema/
  index.md
```

### Markdown conventions

- **Headings:** Use `#` for page title, `##` for sections, `###` for subsections
- **Code blocks:** Specify language for syntax highlighting (` ```bash `, ` ```typescript `, etc.)
- **Tabs:** Use `::: tabs` containers with `== tab "Label"` entries for alternative instructions (for example, package manager commands)
- **Links:** Use root-relative paths for internal links, for example `[link text](/guide/getting-started/)`
- **Line length:** Keep lines under 100 characters

### Frontmatter

Every documentation page should include YAML frontmatter at the top:

```yaml
---
title: Page title
description: One sentence for SEO and search results.
---
```

### Internal links

```markdown
[Getting started guide](/guide/getting-started/)
```

### External links

External links open in a new tab automatically.

## Adding new guide pages

1. Create a new `.md` file in `docs/guide/` (or the relevant subdirectory).
2. Add the required YAML frontmatter with a `title` field.
3. Add the page to the `navigation` array in `docmd.config.js`.
4. Run `pnpm docs:dev` and verify your page appears and renders correctly.
5. Submit a pull request following the [main Contributing Guide](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Documentation commands reference

| Command              | Purpose                                         |
| -------------------- | ----------------------------------------------- |
| `pnpm docs:dev`      | Start local development server with live reload |
| `pnpm docs:build`    | Build production-ready static files             |
| `pnpm docs:validate` | Check all internal links for broken targets     |

## Deployment

Documentation is deployed to Cloudflare Pages at `https://intl-ai.pages.dev/` when changes are merged to the `main` branch.

The deployment process:

1. Detects changes to the `docs/` directory
2. Runs `pnpm docs:build` to generate static files
3. Deploys the output to Cloudflare Pages

No manual deployment steps are required.

## Before submitting documentation changes

- Run `pnpm docs:dev` and verify all pages render correctly
- Run `pnpm docs:validate` to check for broken links
- Check that all code examples are complete and tested
- Follow the writing style rules in `AGENTS.md` (sentence case, no em-dashes, no emoji)

## Code of conduct

Please read and follow our [Code of Conduct](https://github.com/sigilco/intl-ai/blob/main/CODE_OF_CONDUCT.md).

## Questions or issues?

- [Open an issue](https://github.com/sigilco/intl-ai/issues) for documentation bugs or suggestions
- [Start a discussion](https://github.com/sigilco/intl-ai/discussions) for questions
