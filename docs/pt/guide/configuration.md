---
title: Configuração
description: Referência do ficheiro de configuração intl-ai. intl-ai.toml, .json ou .yaml, validado contra um esquema publicado.
---

# Configuração

intl-ai lê um único arquivo de configuração, `intl-ai.toml`, `intl-ai.json`, `intl-ai.yaml` ou `intl-ai.yml`, descoberto na raiz do projeto (primeiro encontrado vence). Passe `--config <path>` para escolher explicitamente um, ou `--config -` para ler TOML a partir da entrada padrão.

O contrato é validado contra um JSON Schema publicado em `https://intl-ai.illo.fyi/schema/v1.json` (`intl-ai config schema` imprime o mesmo esquema).

## Configuração mínima

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

## Campos obrigatórios

### `locale_dir`

Diretório contendo os arquivos de localidade, resolvido em relação ao diretório do arquivo de configuração. Um arquivo por localidade: `${locale_dir}/${locale}.json` (ou `.yaml`).

### `source`

Código de localidade de origem, por exemplo `"en"`.

### `targets`

Códigos de localidade alvo para preencher e verificar.

### `[provider]`

Como as traduções são produzidas. Três tipos:

- `kind = "http"`: um endpoint de chat-completions compatível com OpenAI (`provider`/`model`/`api_key`/`base_url`/`model_params`). Consulte [Providers](/pt/guide/providers/).
- `kind = "command"`: um agente CLI local (`command`/`args`/`agent`/`prompt_via`/`cwd`/`timeout_ms`). Permitido apenas no ficheiro de configuração raiz, nunca via `extends`.
- `kind = "replay"`: um ficheiro JSON de cassette de traduções pré-definidas, para testes e demonstrações.

## Campos optionais

### Interpolação de `api_key`

Os segredos são interpolados ao carregar: `${env:VAR}` lê uma variável de ambiente, `${file:PATH}` lê um ficheiro. `config validate` imprime-os mascarados.

```toml
api_key = "${env:OPENAI_API_KEY}"
```

### `glossary`

Pares fixos de termo para tradução injetados em cada prompt de tradução.

```toml
[glossary]
React = "React"
TypeScript = "TypeScript"
```

### `locale_instructions`

Instrução de estilo ou dialeto livre por localidade, enviada ao modelo e à verificação do juiz. Ordem de resolução: localidade exata (`en-GB`), depois subtag de idioma (`en`), depois `*` como catch-all.

```toml
[locale_instructions]
en-GB = "Use British spelling (colour, organise)."
"*" = "Keep a formal tone."
```

Alterar uma instrução não retraduz as entradas existentes. Retraduzir uma locale com `intl-ai fill --locale en-GB --regenerate`.

### `batch_size`

Entradas máximas de origem por pedido de tradução. Por defeito é ilimitado (todas as chaves em um lote). Reduza para modelos com janelas de contexto menores ou falhas mais granulares por chave.

### `max_retries`

Tentativas de transporte por pedido antes de o lote falhar. Predefinido `3`, limitado a `10`.

### `processor`

Dica de contrato de placeholder enviada ao modelo. `icu` seleciona ICU MessageFormat; o padrão é passthrough.

### `format`

Formato de ficheiro de locale criado para novos ficheiros: `json` (por defeito), `yaml` ou um nome `[[formats]]`. A extensão de um ficheiro existente sempre ganha, então directórios misturados funcionam. Nomes desconhecidos falham a validação.

### `extends`

Configs de composição: uma configuração raiz pode estender uma base (lista ou caminho único). Ficheiros estendidos não podem definir `provider.command`, checks `exec` ou formatos `exec`: ficheiros de dados de uma dependência não devem gerar programas.

## Formatos personalizados

### `[[formats]]`

Formatos de locale externos registados ao lado dos incorporados. Consulte [Format plugins](/pt/guide/format-plugins/) para o protocolo e uma implementação de referência.

```toml
[[formats]]
name = "xml"
exec = "python3"
args = ["tools/strings_xml.py"]
extension = "xml"
extensions = ["axml"]     # optional extra claimed extensions
cwd = "tools"            # optional; defaults to the config directory
timeout_ms = 60000       # optional
max_stdout_bytes = 10485760
```

## Verificações e portas de qualidade

### `[[checks]]`

Verificações de validação executadas por `intl-ai check` e a porta de preenchimento.

```toml
[[checks]]
id = "icu"

[[checks]]
id = "placeholder-parity"

[[checks]]
id = "judge"
threshold = 0.85
weight = 2.0
```

Builtin ids: `icu`, `placeholder-parity`, `dialect:<locale>`, `judge`. Uma entrada de verificação também pode apontar para uma especificação YAML declarativa (`spec = "checks/brand.yaml"`) ou para um comando externo (`exec = "./my-check"` com `args`/`cwd`/`timeout_ms`/`max_stdout_bytes`, apenas config raiz). `threshold` é apenas do juiz (pontuação abaixo dele se torna uma descoberta `invalid`, padrão `0.8`); `weight` alimenta o agregado `[quality]`.

### `[check]`

```toml
[check]
fail_on = ["missing", "stale", "invalid"]
cache = true
```

`fail_on` lista os tipos de descoberta que fazem falhar `intl-ai check`. `cache` altera o cache incremental de descobertas (`.intl-ai/check-cache.json`).

### `[fill]`

```toml
[fill]
validate = ["icu", "placeholder-parity", "judge"]
```

A porta shift-left: estas verificações correm dentro do `fill` entre tradução e adoção. Apenas verificações elegíveis para feedback qualificam (`icu`, `placeholder-parity`, `judge`). Chaves falhas recebem uma rodada corretiva; chaves não resolvidas são adotadas de qualquer forma, gravadas sob `quality.unresolved` no lockfile, e `check` continua a sinalizá-las. Substitua por execução com `--validate`/`--no-validate`.

### `[quality]`

```toml
[quality]
fail_below = 0.5
review_below = 0.8
```

Pontuação ponderada por chave sobre as verificações configuradas (verificações binárias pontuam 1.0/0.0, o juiz pontua seu valor real 0..1). Abaixo de `fail_below` há uma descoberta `invalid`; abaixo de `review_below` marca a chave como não revisada.

## Intellisense do editor

Adicione `"$schema": "https://intl-ai.illo.fyi/schema/v1.json"` a um config JSON para autocompletar e validação.

## Validação CI

```bash
intl-ai config validate   # resolves extends, masks secrets, fails on bad shape
```

Consulte [Providers](/pt/guide/providers/) para tipos de provedor e [AI model setup](/pt/guide/ai-model/) para seleção de endpoint.
