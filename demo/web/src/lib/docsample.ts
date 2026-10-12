export const SAMPLE_DOC = `---
title: Getting started with intl-ai
description: Translate your locale files at build time
---

# Getting started

intl-ai fills the missing keys in your locale files through any
OpenAI-compatible provider, then validates the result.

## Install it

Run the installer, or grab the binary from your package manager:

\`\`\`sh
curl -fsSL https://intl-ai.illo.fyi/install.sh | bash
intl-ai fill --target fr
\`\`\`

## How it works

The pipeline flattens your catalog, batches the missing keys, and writes
the translated file back. A lockfile tracks whether each entry came from
a human or the model, so re-runs stay incremental.

- Keys already translated are never touched
- \`fill --validate\` re-prompts until output is ICU-clean
- \`check\` fails CI on missing or stale entries

| Command | Purpose |
| ------- | ------- |
| \`fill\`  | Translate missing keys |
| \`check\` | Validate the catalog   |

Read the [configuration guide](/guide/configuration) for the full
\`intl-ai.toml\` schema.
`;
