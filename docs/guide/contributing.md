---
title: Contributing
description: Contribute to intl-ai documentation. Edit any doc page on GitHub and open a pull request.
---

# Contributing to the documentation

Thank you for your interest in improving the intl-ai documentation. This guide covers everything you need to know about contributing to the docs.

For general contribution guidelines, branch workflows, commit conventions, and the full development process, see our [main Contributing Guide](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Documentation setup

The docs are built with [VitePress](https://vitepress.dev/), a static site generator optimized for technical documentation.

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

This starts a local dev server (typically at `http://localhost:5173`) with hot module replacement for instant preview updates.

## Building for production

```bash
pnpm docs:build
```

This generates optimized static files in the `docs/.vitepress/dist/` directory.

## Previewing the production build

After building, preview the production version locally:

```bash
pnpm docs:preview
```

## Documentation writing guidelines

### File structure

```
docs/
  .vitepress/
    config.ts
  guide/
    getting-started.md
    ai-model.md
    configuration.md
    api.md
    migration.md
    contributing.md
    next-js.md
    vue-i18n.md
    i18next.md
    mobile/
      expo.md
      flutter.md
      swiftui.md
      jetpack.md
    desktop/
      dotnet.md
  public/
    logo.svg
  index.md
```

### Markdown conventions

- **Headings:** Use `#` for page title, `##` for sections, `###` for subsections
- **Code blocks:** Specify language for syntax highlighting (` ```bash `, ` ```typescript `, etc.)
- **Links:** Use relative paths for internal links: `[link text](/guide/page-name)`
- **Line length:** Keep lines under 100 characters

### Frontmatter

Every documentation page must include YAML frontmatter at the top:

```yaml
---
title: Page title
---
```

### Internal links

```markdown
[Getting started guide](/guide/getting-started)
```

### External links

External links open in a new tab automatically.

## Adding new guide pages

1. Create a new `.md` file in `docs/guide/`.
2. Add the required YAML frontmatter with a `title` field.
3. Add the page to the sidebar in `docs/.vitepress/config.ts`.
4. Run `pnpm docs:dev` and verify your page appears and renders correctly.
5. Submit a pull request following the [main Contributing Guide](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Documentation commands reference

| Command             | Purpose                                        |
| ------------------- | ---------------------------------------------- |
| `pnpm docs:dev`     | Start local development server with hot reload |
| `pnpm docs:build`   | Build production-ready static files            |
| `pnpm docs:preview` | Preview the production build locally           |

## Deployment

Documentation is deployed to Cloudflare Pages at `https://intl-ai.pages.dev/` when changes are merged to the `main` branch.

The deployment process:

1. Detects changes to the `docs/` directory
2. Runs `pnpm docs:build` to generate static files
3. Deploys the output to Cloudflare Pages

No manual deployment steps are required.

## Before submitting documentation changes

- Run `pnpm docs:dev` and verify all pages render correctly
- Check that internal links work (no 404s)
- Verify code examples are accurate and runnable
- Ensure frontmatter is present on all new pages
- Update the sidebar navigation if adding new pages
- Follow Markdown conventions and style guidelines
- Proofread for spelling and grammar

## Code of conduct

All contributors must adhere to our [Code of Conduct](https://github.com/sigilco/intl-ai/blob/main/CODE_OF_CONDUCT.md). We are committed to providing a welcoming and inclusive environment for all contributors.

## Questions or issues?

- Documentation questions: Open a GitHub Discussion
- Found a typo or error: Open an issue or submit a PR
- Feature suggestions: Open an issue to discuss before implementing
