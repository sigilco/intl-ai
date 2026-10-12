---
title: Migrando de 0.4.x
description: Caminho de atualização das pacotes TypeScript 0.4.x para o binário Rust intl-ai e shims @intl-ai/*.
---

# Migrando de 0.4.x

O intl-ai 1.x substitui os pacotes TypeScript por um único binário Rust. Instale-o através do Homebrew, mise, do script de instalação ou do wrapper npm `intl-ai`; não é necessário Node.js para o executar. Os pacotes `@intl-ai/*` são apenas adaptadores (shims) que localizam e lançam o binário dentro da sua ferramenta de compilação.

Esta página mapeia cada conceito da versão 0.4.x para o seu equivalente na 1.x.

## Pacotes

| 0.4.x               | 1.x                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------ |
| `@intl-ai/cli`      | `intl-ai` binário (brew, mise, install.sh, or npm wrapper)                           |
| `@intl-ai/api`      | Removido; o binário é toda a superfície da API                                       |
| `@intl-ai/unplugin` | `@intl-ai/unplugin` shim (iniciar o binário em `buildStart`)                         |
| `@intl-ai/next`     | `@intl-ai/next` shim (`withIntlAi()` executa o binário na avaliação da configuração) |

`@intl-ai/cli` e `@intl-ai/api` permanecem no npm, descontinuados nas últimas versões 0.x. Se usou `@intl-ai/api` programaticamente em TypeScript, migre para executar o binário (`intl-ai fill`, `intl-ai check`) ou para configurações de provedor `kind = "command"`.

## Instalar

```bash
brew install sigilco/intl-ai/intl-ai
# or
mise use -g intl-ai
# or the npm wrapper, which downloads the platform binary on install
npm install -D intl-ai
```

## Configuração

0.4.x lê `intl-ai.config.{ts,js,mjs}` ou `.intl-ai.json` com chaves camelCase. 1.x lê `intl-ai.toml`, `intl-ai.json`, `intl-ai.yaml`, ou `intl-ai.yml` com chaves snake_case.

### Mapeamento de campos

| 0.4.x                       | 1.x                                                       |
| --------------------------- | --------------------------------------------------------- |
| `defaultLocale`             | `source`                                                  |
| `locales` (exceto o padrão) | `targets`                                                 |
| `localeDir`                 | `locale_dir`                                              |
| `glossary`                  | `glossary`                                                |
| `localeInstructions`        | `locale_instructions`                                     |
| `maxRetries`                | `max_retries`                                             |
| `batchSize`                 | `batch_size`                                              |
| `processor`                 | `processor` (padrão `passthrough`, `icu`)                 |
| `format`                    | `format` (`json` padrão, `yaml`)                          |
| `provider`, `model`         | `[provider]` `provider`, `model`                          |
| `apiKey`                    | `api_key` (interpolar segredos, veja abaixo)              |
| `baseURL`                   | `base_url`                                                |
| `modelParams`               | `model_params`                                            |
| `kind = "agent"` + `agent`  | `[provider]` `kind = "command"` + `agent`                 |
| `quality.threshold`         | `[[checks]] id = "judge"` `threshold` + `[quality]` bands |

### Exemplo

0.4.x:

```json
{
  "defaultLocale": "en",
  "locales": ["en", "es", "fr"],
  "localeDir": "locales",
  "provider": "openai",
  "model": "gpt-4o",
  "apiKey": "sk-..."
}
```

1.x:

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
model = "gpt-4o"
api_key = "${env:OPENAI_API_KEY}"
```

::: warning
Não cole chaves de API no `intl-ai.toml`. Use interpolação `${env:VAR}` ou `${file:PATH}`; `config validate` imprime segredos mascarados.
:::

## Lockfile

A versão 0.4.x escrevia a proveniência num único `intl-ai.lock.json`. A 1.x escreve ficheiros `intl-ai.lock.d/<locale>.toml` fragmentados. Não existe nenhum comando `intl-ai migrate`: as migrações são fornecidas como documentação que executa manualmente ou entrega a um agente. Na primeira execução, as chaves existentes nos seus ficheiros de localização são registadas como pertencentes a um humano: elas nunca são sobrescritas ou re-traduzidas. Apenas as chaves em falta recebem preenchimentos de IA. Isso significa que a migração prática é:

1. Instale o binário e traduza sua configuração como acima.
2. `intl-ai fill` uma vez; escreve shards `intl-ai.lock.d/` frescas e preenche apenas o que falta.
3. Apague `intl-ai.lock.json`.

Se você confiou anteriormente no rastreamento de origem de IA para re‑traduzir fontes alteradas (`--stale` semantics), essas chaves são de propriedade humana após a migração; use `intl-ai fill --regenerate` ou `--include-human` seletivamente para re‑traduzir uma única vez.

## Comandos

| 0.4.x                  | 1.x                                                     |
| ---------------------- | ------------------------------------------------------- |
| `intl-ai fill`         | `intl-ai fill` (`--dry-run` préviews)                   |
| `intl-ai fill --force` | `intl-ai fill --regenerate` / `--include-human`         |
| `intl-ai check`        | `intl-ai check` (`--fail-on` para limites de gravidade) |
| —                      | `intl-ai init`, `status`, `config validate`             |

## Validação e qualidade

O 0.4.x `quality.threshold` agora é um sistema de verificação de primeira classe: `[[checks]]` entradas para `icu`, `placeholder-parity`, `dialect:<locale>`, e `judge` (com per-cheque `threshold`/`weight`), agregadas por `[quality]` `fail_below`/`review_below` bandas em pass/review/fail por chave. Veja [Configuration](/pt/guide/configuration/) e [Observability](/pt/guide/observability/).

## Adaptadores (Shims)

Se ligou o intl-ai a um bundler, os pacotes shim mudaram de forma:

- `@intl-ai/unplugin`: mesmo nome, mas as opções agora descrevem o comportamento binário (`fill`, `failOn`, `validate`, `judgeThreshold`, `dev`, `strict`, `config`, `cwd`, `bin`, `args`).
- `@intl-ai/next`: `withIntlAi(nextConfig, options)` executa o binário durante a avaliação da configuração, portanto funciona tanto no webpack quanto no Turbopack.
- `@intl-ai/expo`: plugin de configuração que executa o binário durante `prebuild`.

Todos os adaptadores (shims) declaram `intl-ai` como dependência para que o binário se resolva dentro de projetos instalados via npm.
