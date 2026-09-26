import { NextConfig } from "next";
import createNextIntlPlugin from "next-intl/plugin";
import withIntlAi from "@intl-ai/next";

const withConfigProcessors = async (
  _: NextConfig,
  ...processors: ((nextConfig?: NextConfig) => NextConfig | Promise<NextConfig>)[]
) => {
  let current = _;

  for (const processor of processors) {
    current = await processor(current);
  }

  return current;
};

const withNextIntl = createNextIntlPlugin();
const withNextIntlAi = withIntlAi({ debug: true });

const config: NextConfig = {};

export default withConfigProcessors(config, withNextIntlAi, withNextIntl);
