---
title: Configuração de documentos
description: Referência de intl-ai-docs.toml. Experimental.
---

# Configuração de documentos

O `intl-ai-docs` lê o seu próprio ficheiro de configuração, `intl-ai-docs.toml` (ou `.json`/`.yaml`), descoberto percorrendo para cima a partir do diretório de trabalho. Coloca-o junto da árvore de documentos que gere; está deliberadamente separado do `intl-ai.toml` para que os pipelines de catálogos e de documentos nunca partilhem definições.

Tudo o que no `intl-ai.toml` não tem forma de documento (ficheiros de locale, shims, checks de corpus) fica lá. Os arms de fornecedor são idênticos aos da configuração de corpus: qualquer coisa de [Fornecedores](/pt/guide/providers/) funciona tal como está no `intl-ai-docs.toml`.

## Nível superior

| Key                   | Type   | Default  | Description                                                                                                                                 |
| --------------------- | ------ | -------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `experimental`        | bool   | `false`  | Obrigatório. Todos os comandos recusam executar até ser `true`.                                                                             |
| `layout`              | string | `"flat"` | Disposição da árvore: `locale-dirs` coloca a fonte em `source_dir/<source>`; `flat` coloca os ficheiros fonte diretamente sob `source_dir`. |
| `source_dir`          | path   | `"docs"` | Raiz dos documentos, resolvida em relação ao ficheiro de configuração.                                                                      |
| `source`              | string | `"en"`   | Id da locale fonte.                                                                                                                         |
| `targets`             | list   | required | Locales de destino; cada uma escreve em `source_dir/<locale>`.                                                                              |
| `format`              | string | `"mdx"`  | Nome do DocFormat. `"mdx"` (markdown incluído) é o único formato publicado; `"markdown"` é um alias.                                        |
| `batch_size`          | int    | engine   | Segmentos por pedido de tradução. Mantém pequeno para documentos longos para não exceder os limites de saída do fornecedor.                 |
| `max_retries`         | int    | `3`      | Tentativas do fornecedor em falhas repetíveis.                                                                                              |
| `glossary`            | map    | `{}`     | Termos fixados, com a mesma semântica do glossário de corpus.                                                                               |
| `locale_instructions` | map    | `{}`     | Notas de estilo/dialeto por locale; resolvidas por locale exata, subtag de idioma e depois `*`.                                             |
| `exclude`             | list   | `[]`     | Globs extra (relativos ao documento) a ignorar, além de diretórios ocultos, `node_modules` e os diretórios das locales de destino.          |
| `version`             | int    | —        | Versão do esquema de configuração; apenas `1` é compreendido.                                                                               |

## `[provider]`

Mesma forma que `intl-ai.toml`: `kind = "http" | "command" | "replay"`, mais os campos específicos do fornecedor. `api_key` aceita a interpolação `${env:VAR}` e `${file:PATH}`.

## `[links]`

| Key       | Type   | Default       | Description                                                      |
| --------- | ------ | ------------- | ---------------------------------------------------------------- |
| `rewrite` | bool   | `true`        | Insere o prefixo da locale nos destinos de ligação com raiz `/`. |
| `prefix`  | string | `"/{locale}"` | Modelo de prefixo; `{locale}` é substituído por cada destino.    |

As ligações são reescritas apenas no merge; os shards guardam a forma sem prefixo, por isso mudar `prefix` mais tarde reescreve os destinos de forma limpa no próximo fill.

## `[frontmatter]`

| Key    | Type | Default                    | Description                                                               |
| ------ | ---- | -------------------------- | ------------------------------------------------------------------------- |
| `keys` | list | `["title", "description"]` | Caminhos de frontmatter (com pontos) cujos valores escalares se traduzem. |

## `[fill]`

| Key        | Type | Default | Description                                                                                        |
| ---------- | ---- | ------- | -------------------------------------------------------------------------------------------------- |
| `validate` | list | `[]`    | Checks de documento executados como porta shift-left por lote: `doc:protected` e `doc:corruption`. |

## `[check]` e `[[checks]]`

`[check]` leva as mesmas opções do check de corpus: `fail_on` (tipos de achado que saem com código diferente de zero, por defeito `missing`, `stale`, `invalid`) e `cache` (cache incremental de achados, por defeito `true`). `[[checks]]` acrescenta entradas estilo corpus spec/exec/judge além dos builtins de documento sempre ativos, que verificam a paridade do esqueleto, as regiões protegidas, os prefixos de ligações e a consistência do rekey.

## `[quality]`

Mesmo bloco de LLM-as-a-judge que `intl-ai.toml` (`enabled`, `provider`, limiares). As pontuações ficam nos shards do lockfile sob `quality`.

## Lockfile

`intl-ai-docs.lock.d/` fica junto do ficheiro de configuração. Cada documento traduzido tem um shard em `intl-ai-docs.lock.d/<locale>/<relpath>.toml` com `sourceHash`, `origin` (`ai` ou `human`), `reviewed` e `quality` por segmento. Também vive lá um `check-cache.json` incremental; adiciona-o ao gitignore.
