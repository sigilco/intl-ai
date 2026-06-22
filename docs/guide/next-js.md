---
title: Next.js (Turbopack)
---

# Next.js (Turbopack)

Framework-specific setup for Next.js applications with Turbopack support and internationalization integration.

## Installation

Install the `@intl-ai/next` package in your Next.js project:

::: code-group

```sh [npm]
npm install @intl-ai/next
```

```sh [pnpm]
pnpm add @intl-ai/next
```

```sh [yarn]
yarn add @intl-ai/next
```

:::

This package provides a Higher-Order Function (HOF) that integrates `intl-ai` with your Next.js configuration, supporting both Turbopack and webpack bundlers.

## Turbopack Support

### Turbopack vs Webpack

The `@intl-ai/next` package automatically handles both Turbopack and webpack:

- **Turbopack**: Runs configuration evaluation at build time. The `withIntlAi` HOF applies `turbopack.rules` to configure the loader for processing internationalization files.
- **Webpack**: Continues to work seamlessly with the same configuration approach.

Both bundlers are supported automatically, with no additional configuration needed to switch between them. Next.js 16+ with Turbopack is the primary target, but the package maintains full compatibility with webpack-based setups.

### How It Works

When you apply the `withIntlAi` HOF to your Next.js config, it:

1. Detects your bundler (Turbopack or webpack)
2. Applies the appropriate loader configuration
3. Processes internationalization files during the build phase
4. Maintains compatibility with other Next.js plugins

## Configuration

### Basic Setup with next.config.ts

Create or update your `next.config.ts` file:

```typescript
import { NextConfig } from "next";
import createNextIntlPlugin from "next-intl/plugin";
import withIntlAi from "@intl-ai/next";

const withConfigProcessors = async (
  _: NextConfig,
  ...processors: ((nextConfig?: NextConfig) => Promise<NextConfig>)[]
) => {
  let current = _;
  for (const processor of processors) {
    current = await processor(current);
  }
  return current;
};

const withNextIntl = createNextIntlPlugin();
const withNextIntlAi = withIntlAi({ debug: true });

export default withConfigProcessors(config, withNextIntlAi, withNextIntl);
```

### Configuration Options

The `withIntlAi` HOF accepts the following options:

```typescript
interface WithIntlAiOptions {
  debug?: boolean; // Enable debug logging (default: false)
}
```

- **debug**: Set to `true` to enable detailed logging during the build process. Useful for troubleshooting loader configuration and file processing.

### Integration with next-intl

The example above shows how to combine `@intl-ai/next` with `next-intl` using a config processor pattern:

1. Create a `withConfigProcessors` helper that chains multiple HOFs
2. Apply `withNextIntlAi` first (intl-ai configuration)
3. Apply `withNextIntl` second (next-intl plugin)

This order ensures that intl-ai processes files before next-intl applies its own transformations.

## App Router

### Setup

The App Router is fully supported with `@intl-ai/next`. Once your `next.config.ts` is configured, you can use intl-ai in your app directory:

```typescript
// app/layout.tsx
import { NextIntlClientProvider } from "next-intl";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html>
      <body>
        <NextIntlClientProvider>
          {children}
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
```

### Using Translations in Components

Access translations in your Server and Client Components:

```typescript
// app/page.tsx (Server Component)
import { getTranslations } from "@intl-ai/next";

export default async function Page() {
  const t = await getTranslations();

  return <h1>{t("home.title")}</h1>;
}
```

```typescript
// app/components/greeting.tsx (Client Component)
"use client";

import { useTranslations } from "next-intl";

export function Greeting() {
  const t = useTranslations();

  return <p>{t("greeting.message")}</p>;
}
```

## Pages Router

### Setup

The Pages Router is also fully supported. Configure your `next.config.ts` as shown above, then use intl-ai in your pages:

```typescript
// pages/_app.tsx
import { NextIntlClientProvider } from "next-intl";
import type { AppProps } from "next/app";

export default function App({ Component, pageProps }: AppProps) {
  return (
    <NextIntlClientProvider>
      <Component {...pageProps} />
    </NextIntlClientProvider>
  );
}
```

### Using Translations in Pages

```typescript
// pages/index.tsx
import { getTranslations } from "@intl-ai/next";

export default function Home({ translations }: { translations: Record<string, string> }) {
  return <h1>{translations["home.title"]}</h1>;
}
```

## Troubleshooting

Enable debug mode (`withIntlAi({ debug: true })`) to diagnose issues.

## Next Steps

- [Explore the intl-ai documentation](/guide/getting-started)
- [Learn about configuration options](/guide/configuration)
