---
"@intl-ai/core": major
---

Deprecate `@intl-ai/core`. The package now prints a console warning on first import directing users to `@intl-ai/api`. The package will be removed in v1.0.

Migration:
- `import { runFill } from "@intl-ai/core"` → `import { runFill } from "@intl-ai/api"`
- `import { loadConfig } from "@intl-ai/core"` → read the JSON config file directly with `IntlAiJsonConfigSchema.parse(JSON.parse(text))` and `jsonConfigToIntlAiConfig()`
- See https://github.com/sigilco/intl-ai/blob/main/docs/migration.md
