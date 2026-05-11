---
title: Contributing
---

# Contributing to the Documentation

Thank you for your interest in improving the unplugin-intl-ai documentation! This guide covers everything you need to know about contributing to our docs.

For general contribution guidelines, branch workflows, commit conventions, and the full development process, please see our [main Contributing Guide](https://github.com/OWNER/unplugin-intl-ai/blob/main/CONTRIBUTING.md).

## Documentation Setup

Our documentation is built with [VitePress](https://vitepress.dev/), a static site generator optimized for technical documentation.

### Prerequisites

- Node.js 22+ and pnpm 10+
- Basic familiarity with Markdown
- A text editor (VS Code recommended)

### Installation

```bash
# Install dependencies
pnpm install

# Verify VitePress is installed
pnpm docs:dev --version
```

## Running Docs Locally

Start the development server to preview your changes in real-time:

```bash
pnpm docs:dev
```

This command:

- Starts a local dev server (typically at `http://localhost:5173`)
- Enables hot module replacement (HMR) for instant preview updates
- Watches for file changes automatically

The dev server will remain running until you stop it with `Ctrl+C`.

## Building for Production

To create a production-ready build:

```bash
pnpm docs:build
```

This generates optimized static files in the `docs/.vitepress/dist/` directory.

## Previewing the Production Build

After building, preview the production version locally:

```bash
pnpm docs:preview
```

This serves the built files and helps verify the production build works correctly before deployment.

## Documentation Writing Guidelines

### File Structure

Documentation files are organized in the `docs/` directory:

```
docs/
├── .vitepress/
│   └── config.ts          # VitePress configuration
├── index.md               # Home page (hero)
└── guide/
    ├── getting-started.md # Getting started guide
    ├── ai-model.md        # AI model setup
    ├── configuration.md   # Configuration reference
    ├── next-js.md         # Next.js integration
    └── contributing.md    # This file
```

### Markdown Conventions

- **Headings**: Use `#` for page title, `##` for sections, `###` for subsections
- **Code blocks**: Specify language for syntax highlighting (e.g., ` ```bash `)
- **Links**: Use relative paths for internal links: `[link text](/guide/page-name)`
- **Line length**: Keep lines under 100 characters for readability
- **Lists**: Use `-` for unordered lists, `1.` for ordered lists

### Frontmatter

Every documentation page must include YAML frontmatter at the top:

```yaml
---
title: Page Title
---
```

The `title` is used in the sidebar navigation and browser tab.

### Code Examples

Include practical, runnable examples:

```bash
# Good: Clear, complete example
pnpm docs:dev

# Bad: Incomplete or unclear
run docs
```

For multi-step examples, use numbered lists:

1. First step with code block
2. Second step with code block
3. Verify the result

### Internal Links

Link to other documentation pages using relative paths:

```markdown
# Correct

See the [Getting Started guide](/guide/getting-started)

# Incorrect

See the [Getting Started guide](getting-started.md)
```

### External Links

External links open in a new tab automatically:

```markdown
[VitePress Documentation](https://vitepress.dev/)
```

## Adding New Guide Pages

To add a new guide page:

### 1. Create the Markdown File

Create a new `.md` file in `docs/guide/`:

```bash
touch docs/guide/your-page-name.md
```

### 2. Add Frontmatter

Add the required YAML frontmatter:

```yaml
---
title: Your Page Title
---
# Your Page Title

Your content here...
```

### 3. Update the Sidebar Navigation

Edit `docs/.vitepress/config.ts` and add your page to the sidebar:

```typescript
sidebar: {
  "/guide/": [
    { text: "Getting Started", link: "/guide/getting-started" },
    { text: "AI Model Setup", link: "/guide/ai-model" },
    { text: "Configuration", link: "/guide/configuration" },
    { text: "Next.js (Turbopack)", link: "/guide/next-js" },
    { text: "Your New Page", link: "/guide/your-page-name" },  // Add here
    { text: "Contributing", link: "/guide/contributing" },
  ],
},
```

### 4. Test Locally

Run the dev server and verify your page appears in the sidebar and renders correctly:

```bash
pnpm docs:dev
```

### 5. Submit a Pull Request

Create a PR with your new page following the [main Contributing Guide](https://github.com/OWNER/unplugin-intl-ai/blob/main/CONTRIBUTING.md).

## Documentation Commands Reference

| Command             | Purpose                                        |
| ------------------- | ---------------------------------------------- |
| `pnpm docs:dev`     | Start local development server with hot reload |
| `pnpm docs:build`   | Build production-ready static files            |
| `pnpm docs:preview` | Preview the production build locally           |

## Deployment

Documentation is automatically deployed to GitHub Pages via GitHub Actions when changes are merged to the `main` branch.

The deployment workflow:

1. Detects changes to the `docs/` directory
2. Runs `pnpm docs:build` to generate static files
3. Deploys to GitHub Pages at `https://OWNER.github.io/unplugin-intl-ai/`

No manual deployment steps are required.

## Before Submitting Documentation Changes

- [ ] Run `pnpm docs:dev` and verify all pages render correctly
- [ ] Check that internal links work (no 404s)
- [ ] Verify code examples are accurate and runnable
- [ ] Ensure frontmatter is present on all new pages
- [ ] Update the sidebar navigation if adding new pages
- [ ] Follow Markdown conventions and style guidelines
- [ ] Proofread for spelling and grammar
- [ ] Keep commit messages clear and descriptive

## Code of Conduct

All contributors must adhere to our [Code of Conduct](https://github.com/OWNER/unplugin-intl-ai/blob/main/CODE_OF_CONDUCT.md). We are committed to providing a welcoming and inclusive environment for all contributors.

## Questions or Issues?

- **Documentation questions**: Open a GitHub Discussion
- **Found a typo or error**: Open an issue or submit a PR
- **Feature suggestions**: Open an issue to discuss before implementing

Thank you for helping improve the unplugin-intl-ai documentation! 🚀
