---
"@intl-ai/core": patch
"@intl-ai/cli": patch
"@intl-ai/unplugin": patch
"@intl-ai/next": patch
---

fix: resolve workspace:* dependencies so published packages install correctly

Published v0.1.0 packages had workspace:* dependencies which fail to resolve for npm consumers. This patch replaces workspace:* with proper semver ranges via changeset version bump.
