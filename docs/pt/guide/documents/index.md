---
title: Localize documentos
description: Traduzir markdown e MDX árvores de documentos com intl-ai-docs. Experimental.
---

# Localize documentos

`intl-ai-docs` é um segundo binário que acompanha cada instalação de `intl-ai`. Ele traduz árvores de documentos completas (markdown e MDX) da mesma forma que `intl-ai` traduz ficheiros de locale: uma árvore fonte num único idioma, uma árvore gerada por locale de destino, e um ficheiro de bloqueio que regista de onde veio cada coisa.

::: warning Experimental
O suporte a documentos é opcional. Todos os comandos `intl-ai-docs` recusam-se a executar a menos que a sua configuração defina `experimental = true`. O leitor MDX processa as construções que os sites de documentação reais usam (frontmatter, cercas de código, callouts, blocos JSX), mas o formato é novo: execute `intl-ai-docs check` no CI antes de publicar.
:::

## Instalar

Já o tem. O script de instalação, a fórmula do Homebrew, o pacote npm e o mise instalam ambos o `intl-ai` e o `intl-ai-docs`. Verifique com `intl-ai-docs --help`.

## Como funciona

Em vez de chaves e valores, `intl-ai-docs` divide cada documento num **skeleton** e **segments**:

1. **Extracção**: o analisador MDX percorre o documento e emite um skeleton (a estrutura do documento com IDs de marcador de posição) mais um segmento por intervalo traduzível: títulos, parágrafos, itens de lista, células de tabela e strings de frontmatter.
2. **Preencher**: os segmentos são traduzidos em lotes pelo seu provedor, exatamente como `intl-ai fill` trata chaves de catálogo.
3. **Fundir**: segmentos traduzidos são inseridos no esqueleto por intervalo de bytes. Estrutura, comentários, cercas de código, código inline e componentes MDX são preservados byte a byte porque nunca estiveram dentro de um segmento.

As regiões protegidas (blocos de código, código inline, comandos, URLs) não podem ser corrompidas por construção: o modelo nunca as vê como texto traduzível, e o portão de escrita verifica a paridade antes de qualquer coisa ser escrita.

## Início rápido

Crie `intl-ai-docs.toml` ao lado da sua árvore de documentação:

```toml
experimental = true
layout = "locale-dirs"      # source at docs/en, targets at docs/<locale>
source_dir = "docs"
source = "en"
targets = ["es", "pt"]
batch_size = 8

[provider]
kind = "http"
provider = "openai"
base_url = "${env:AI_BASE_URL}"
model = "${env:AI_MODEL_ID}"
api_key = "${env:AI_API_KEY}"
```

Depois preencha e verifique:

```bash
intl-ai-docs fill     # writes docs/es/** and docs/pt/**
intl-ai-docs check    # read-only report; --fail-on invalid for CI
```

Documentos traduzidos ficam em `docs/<locale>/` espelhando a árvore de origem. A proveniência vive em `intl-ai-docs.lock.d/<locale>/<doc>.toml` shards com os mesmos campos do lockfile do corpus (`origin`, `sourceHash`, `reviewed`, `quality`). Faça commit tanto as árvores quanto os shards.

## Layouts

`layout` controla onde a árvore fonte vive relativamente aos seus alvos:

- `locale-dirs`: cada locale é um diretório irmão, incluindo a origem (`docs/en`, `docs/es`, `docs/pt`). Isto corresponde ao docmd i18n e a maioria dos frameworks de documentos que querem um diretório por locale.
- `flat`: os ficheiros de origem ficam diretamente sob `source_dir`, e as árvores de destino são escritas em `source_dir/<locale>`. Use isto quando os seus ficheiros de origem não podem residir num diretório de locale.

Os links são reescritos por local por padrão: destinos com prefixo `/` recebem o prefixo do alvo (`/es/guide/...`) para que páginas traduzidas se liguem dentro da sua própria árvore. Ajuste ou desative com `[links]`.

## O que permanece em inglês

- Cercas de código, código inline, comandos, flags, caminhos de ficheiros e URLs.
- Componentes MDX/JSX e sintaxe de callout delimitada; apenas os seus filhos de prosa traduzem.
- Chaves de frontmatter diferentes das listadas em `frontmatter.keys` (padrão `title`, `description`).
- Rótulos da barra lateral: a navegação vive na configuração do site, não nos documentos. Para docmd, coloque um `navigation.json` em cada directório de locale.

Adicione uma entrada `[locale_instructions]` por locale para orientar terminologia, dialecto e registo; a regra de resolução é a mesma que `intl-ai` (locale exato, subtag de idioma, depois `*`).

::: tip Keep backticks verbatim
Os modelos por vezes traduzem dentro de código inline mesmo não devendo. Escreva-o explicitamente em `locale_instructions`, por exemplo: `Never translate text inside backticks; keep code blocks, commands, and identifiers verbatim.`
:::

Veja [Configuração](/pt/guide/documents/configuration/) para a referência completa do `intl-ai-docs.toml`.
