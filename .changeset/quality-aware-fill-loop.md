---
"@intl-ai/api": minor
"@intl-ai/unplugin": minor
"@intl-ai/next": minor
---

feat(fill): quality-aware fill loop with LLM-as-a-Judge retry. `runFill(config, { quality })` now runs an optional quality loop: after each fill batch, the same provider judges the translations, entries below the configured threshold are refilled with the judge's feedback, and a build plugin enabled with `quality: true` fails the build when keys remain below threshold after retries. Threshold and `maxRetries` come from `intl-ai.config.json`; the build plugin option only toggles the loop. Custom judges plug in via `config.quality.assessor: QualityAssessorInstance`. Closes #14.
