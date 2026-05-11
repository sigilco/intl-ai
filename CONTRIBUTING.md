# Contributing to intl-ai

Thank you for your interest in contributing to intl-ai! This document outlines our development workflow, commit conventions, and release process.

## Quick Start

Get your development environment set up:

```bash
# Install dependencies
pnpm install

# Build all packages
pnpm build

# Run tests
pnpm test
```

## Branch Workflow

We follow a structured branching model to maintain code quality and enable smooth releases.

| Branch      | Purpose                                             | Created From | Merges To                        |
| ----------- | --------------------------------------------------- | ------------ | -------------------------------- |
| `main`      | Production releases only                            | N/A          | N/A                              |
| `develop`   | Integration branch for features                     | `main`       | `main` (via release)             |
| `release/*` | Release preparation (e.g., `release/v1.0.0`)        | `develop`    | `main` + back-merge to `develop` |
| `feature/*` | New features (e.g., `feature/add-locale-support`)   | `develop`    | `develop` (via PR)               |
| `fix/*`     | Bug fixes (e.g., `fix/locale-parsing-error`)        | `develop`    | `develop` (via PR)               |
| `chore/*`   | Maintenance, deps, docs (e.g., `chore/update-deps`) | `develop`    | `develop` (via PR)               |

### Branch Naming Conventions

- **Feature branches**: `feature/short-description` (kebab-case)
- **Fix branches**: `fix/short-description` (kebab-case)
- **Chore branches**: `chore/short-description` (kebab-case)
- **Release branches**: `release/vX.Y.Z` (semantic versioning)

## Commit Conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/) to maintain a clear, semantic commit history.

### Commit Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Commit Types

- **feat**: A new feature
- **fix**: A bug fix
- **chore**: Build process, dependencies, tooling (no code change)
- **docs**: Documentation only
- **test**: Adding or updating tests
- **ci**: CI/CD configuration changes
- **refactor**: Code refactoring without feature changes
- **build**: Build system or dependency changes
- **perf**: Performance improvements

### Examples

```bash
# Feature
git commit -m "feat(i18n): add support for RTL languages"

# Bug fix
git commit -m "fix(parser): handle edge case in locale detection"

# Documentation
git commit -m "docs: update setup instructions in README"

# Chore
git commit -m "chore(deps): upgrade typescript to 5.0"

# Test
git commit -m "test(parser): add test cases for RTL locales"
```

### Commit Message Guidelines

- Use imperative mood ("add" not "added" or "adds")
- Don't capitalize the subject line
- Don't end the subject line with a period
- Limit the subject line to 50 characters
- Wrap the body at 72 characters
- Reference issues and PRs in the footer: `Closes #123`

## Pull Request Process

### Before Submitting a PR

1. **Create a feature branch** from `develop`:

   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** with atomic, well-documented commits

3. **Run tests locally**:

   ```bash
   pnpm test
   ```

4. **Lint and format**:

   ```bash
   pnpm lint
   pnpm format
   ```

5. **Build to verify**:
   ```bash
   pnpm build
   ```

### PR Checklist

Before submitting your PR, ensure:

- [ ] Tests pass: `pnpm test`
- [ ] Linting passes: `pnpm lint`
- [ ] Code is formatted: `pnpm format`
- [ ] Build succeeds: `pnpm build`
- [ ] Commit messages follow Conventional Commits
- [ ] Documentation is updated (if applicable)
- [ ] No breaking changes (or clearly documented)
- [ ] PR description explains the "why" not just the "what"

### PR Description Template

```markdown
## Description

Brief description of what this PR does.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Related Issues

Closes #123

## Testing

Describe how you tested these changes.

## Screenshots (if applicable)

Add screenshots for UI changes.
```

## Adding a New Bundler Example

To add a new bundler example to the monorepo:

1. **Create the package directory**:

   ```bash
   mkdir -p packages/bundler-name
   cd packages/bundler-name
   ```

2. **Initialize package.json**:

   ```bash
   pnpm init
   ```

3. **Set up the structure**:

   ```
   packages/bundler-name/
   ├── src/
   │   ├── index.ts
   │   └── index.test.ts
   ├── package.json
   ├── tsconfig.json
   └── README.md
   ```

4. **Add to root tsconfig.json** paths:

   ```json
   {
     "compilerOptions": {
       "paths": {
         "@intl-ai/bundler-name": ["packages/bundler-name/src"]
       }
     }
   }
   ```

5. **Update root package.json** workspaces (if using pnpm workspaces)

6. **Create a feature branch** and submit a PR with your new bundler

## Release Process

### Creating a Release

1. **Create a release branch** from `develop`:

   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b release/vX.Y.Z
   ```

2. **Update version numbers**:
   - Update `package.json` version in root and affected packages
   - Update `CHANGELOG.md` with release notes

3. **Commit the version bump**:

   ```bash
   git commit -m "chore(release): bump version to vX.Y.Z"
   ```

4. **Create a PR** from `release/vX.Y.Z` to `main`

5. **Merge to main** after approval:

   ```bash
   git checkout main
   git pull origin main
   git merge --no-ff release/vX.Y.Z
   ```

6. **Tag the release**:

   ```bash
   git tag -a vX.Y.Z -m "Release version X.Y.Z"
   git push origin main --tags
   ```

7. **CI/CD publishes** the release (automated via GitHub Actions)

8. **Back-merge to develop**:

   ```bash
   git checkout develop
   git pull origin develop
   git merge --no-ff main
   git push origin develop
   ```

9. **Delete the release branch**:
   ```bash
   git branch -d release/vX.Y.Z
   git push origin --delete release/vX.Y.Z
   ```

### Semantic Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes
- **MINOR** (0.Y.0): New features (backward compatible)
- **PATCH** (0.0.Z): Bug fixes (backward compatible)

## Code Style

- **TypeScript**: Strict mode enabled
- **Formatting**: Prettier (configured in `.prettierrc`)
- **Linting**: ESLint (configured in `.eslintrc`)
- **Testing**: Jest with >80% coverage target

Run formatting and linting:

```bash
pnpm format
pnpm lint --fix
```

## Testing

All code changes should include tests:

```bash
# Run all tests
pnpm test

# Run tests in watch mode
pnpm test --watch

# Run tests with coverage
pnpm test --coverage
```

Test files should be colocated with source files:

```
src/
├── utils.ts
└── utils.test.ts
```

## Documentation

- Update `README.md` for user-facing changes
- Add JSDoc comments to public APIs
- Update `CHANGELOG.md` for notable changes
- Keep inline comments for complex logic

## Getting Help

- **Questions**: Open a discussion in GitHub Discussions
- **Bugs**: Open an issue with reproduction steps
- **Features**: Open an issue to discuss before implementing

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions.

## Local Development with Verdaccio

For testing intl-ai changes in external projects without publishing to npm, use a local Verdaccio registry.

### Prerequisites

Install Verdaccio globally:

```bash
pnpm install -g verdaccio
```

### Start the Registry

```bash
verdaccio
```

The registry starts at `http://localhost:4873` by default.

### Publish Packages Locally

From the monorepo root, run:

```bash
pnpm publish:local
```

This builds all packages and publishes them to the local registry.

### Install in an External Project

In your consumer project, create or update `.npmrc`:

```
@intl-ai:registry=http://localhost:4873
//localhost:4873/:_authToken=""
```

Then install intl-ai packages:

```bash
pnpm add @intl-ai/core
```

### Development Loop

1. Make changes in the monorepo
2. Run `pnpm publish:local` to rebuild and publish
3. In the consumer project, run `pnpm update @intl-ai/core`
4. Test your changes

### Watch Mode

For automatic rebuilds on file change:

```bash
pnpm dev:registry
```

Note: You still need to run `pnpm publish:local` after builds to update the registry.

### Port Conflicts

If port 4873 is occupied, edit `verdaccio/config.yaml` to use a different port (e.g., 4874) and update your consumer `.npmrc` accordingly.

---

**Thank you for contributing to intl-ai!** 🚀
