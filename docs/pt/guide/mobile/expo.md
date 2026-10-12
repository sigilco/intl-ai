---
title: Expo
description: Tradução de IA em tempo de compilação para Expo i18n. As traduções ocorrem durante o pré-build, sem tempo de execução.
---

# Expo

`@intl-ai/expo` é um plugin de configuração que executa o binário `intl-ai` enquanto o Expo avalia plugins de configuração durante `expo prebuild`, `eas build` e `expo run:*`. As traduções são escritas em disco antes do Metro empacotar a sua aplicação, portanto não há sobrecarga em tempo de execução.

## Instale

```bash
pnpm add -D @intl-ai/expo
```

O pacote npm `intl-ai` vem como dependência e baixa o binário da plataforma na instalação. Para usar um binário que você instalou de outra forma (brew, mise, script de instalação), defina `INTL_AI_BIN` ou passe `bin`.

## Configurar `app.json`

```json
{
  "expo": {
    "plugins": ["@intl-ai/expo"]
  }
}
```

Com opções:

```json
{
  "expo": {
    "plugins": [
      [
        "@intl-ai/expo",
        {
          "failOn": ["missing", "invalid"],
          "judgeThreshold": 0.9
        }
      ]
    ]
  }
}
```

## Crie `intl-ai.toml`

```toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${OPENAI_API_KEY}"
```

Veja [Configuration](/pt/guide/configuration/) para o esquema completo.

## Executar pré-build

```bash
expo prebuild
```

O plugin regista em ambas as plataformas ios e android e executa o pipeline uma vez, com a raiz do app como diretório de trabalho.

## Opções

| Opção            | Tipo                  | Padrão      | Descrição                                                    |
| ---------------- | --------------------- | ----------- | ------------------------------------------------------------ |
| `fill`           | `boolean`             | `true`      | Executar `intl-ai fill`                                      |
| `failOn`         | `string \| string[]`  | —           | Execute `intl-ai check --fail-on <kinds>` depois de fill     |
| `validate`       | `boolean \| string[]` | `true`      | Gate de validação do tempo de preenchimento                  |
| `judgeThreshold` | `number`              | —           | Limiar de pontuação do juiz dentro da porta de preenchimento |
| `dev`            | `boolean`             | `true`      | `false` pula quando `NODE_ENV !== "production"`              |
| `strict`         | `boolean`             | `true`      | `false` avisa em vez de falhar o pré-build                   |
| `config`         | `string`              | —           | Caminho para `intl-ai.toml`/`.json`/`.yaml`                  |
| `cwd`            | `string`              | raiz do app | Diretório de trabalho para o binário                         |
| `bin`            | `string`              | —           | Caminho explícito para o binário `intl-ai`                   |
| `args`           | `string[]`            | —           | Argumentos adicionais adicionados a `intl-ai fill`           |

Ordem de resolução binária: opção `bin` → variável de ambiente `INTL_AI_BIN` → o pacote npm `intl-ai` → `PATH`.

## Utilização em tempo de execução

Carregue os arquivos JSON gerados diretamente com a sua biblioteca i18n preferida (`i18next`, `react-intl`, etc.). O plugin apenas escreve traduções; não impõe uma API de runtime.

## Exemplo

Veja [`examples/expo`](https://github.com/sigilco/intl-ai/tree/main/examples/expo) para uma aplicação de trabalho completa.
