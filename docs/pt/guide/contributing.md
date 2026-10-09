---
title: Contribuição
description: Contribua para a documentação do intl-ai. Edite qualquer página de documentação no GitHub e abra um pull request.
---

# Contribuir com a documentação

Obrigado pelo seu interesse em melhorar a documentação do intl-ai. Este guia cobre tudo o que precisa saber sobre contribuir para a documentação.

Para diretrizes gerais de contribuição, fluxos de ramificação, convenções de commit e todo o processo de desenvolvimento, veja nosso [Guia de Contribuição Principal](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Configuração da documentação

A documentação é criada com [docmd](https://docmd.io/), um gerador de sites estáticos Markdown-first.

### Pré-requisitos

- Node.js 22+ e pnpm 11+
- Familiaridade básica com Markdown
- Um editor de texto (VS Code recomendado)

### Instalação

```bash
pnpm install
```

## Executando documentação localmente

Inicie o servidor de desenvolvimento para pré-visualizar suas alterações em tempo real:

```bash
pnpm docs:dev
```

Isso inicia um servidor de desenvolvimento local (normalmente em `http://localhost:3000`) com recarregamento ao vivo para atualizações instantâneas de pré-visualização.

## Compilação para produção

```bash
pnpm docs:build
```

Isso gera arquivos estáticos otimizados no diretório `docs/.vitepress/dist/` e copia os ativos de `docs/public/` para a raiz do site.

## Visualizando a construção de produção

Depois de construir, sirva o diretório de saída com qualquer servidor de arquivos estáticos para verificar a saída de produção.

## Diretrizes de escrita de documentação

### Estrutura de ficheiros

```
docmd.config.js     # site config, navigation, plugins
assets/             # images served at /assets/
docs/
  guide/
    getting-started.md
    ai-model.md
    configuration.md
    api.md
    contributing.md
    build-systems/
    i18n-libraries/
    mobile/
    desktop/
  public/           # files copied to the site root on build
    install.sh
    schema/
  index.md
```

### Convenções do Markdown

- **Títulos:** Use `#` para o título da página, `##` para seções, `###` para subseções
- **Blocos de código:** Especifique a linguagem para realce de sintaxe (` ```bash `, ` ```typescript `, etc.)
- **Abas:** Use `::: tabs` contentores com `== tab "Label"` entradas para instruções alternativas (por exemplo, comandos do gestor de pacotes)
- **Links:** Use caminhos relativos à raiz para links internos, por exemplo `[link text](/pt/guide/getting-started/)`
- **Comprimento da linha:** Mantenha as linhas com menos de 100 caracteres

### Frontmatter

Cada página de documentação deve incluir frontmatter YAML no topo:

```yaml
---
title: Page title
description: One sentence for SEO and search results.
---
```

### Links internos

```markdown
[Getting started guide](/guide/getting-started/)
```

### Ligações externas

Links externos abrem em uma nova aba automaticamente.

## Adicionar novas páginas de guia

1. Criar um novo ficheiro `.md` em `docs/guide/` (ou o subdiretório correspondente).
2. Adicione o frontmatter YAML obrigatório com um campo `title`.
3. Adicione a página ao array `navigation` em `docmd.config.js`.
4. Execute `pnpm docs:dev` e verifique se a sua página aparece e é renderizada corretamente.
5. Submeta um pull request seguindo o [Guia de Contribuição principal](https://github.com/sigilco/intl-ai/blob/main/CONTRIBUTING.md).

## Referência de comandos de documentação

| Comando              | Objetivo                                         |
| -------------------- | ----------------------------------------------- |
| `pnpm docs:dev`      | Iniciar o servidor de desenvolvimento local com recarga ao vivo |
| `pnpm docs:build`    | Construa ficheiros estáticos prontos para produção             |
| `pnpm docs:validate` | Verifique todas as ligações internas para destinos quebrados     |

## Implantação

A documentação é implementada no Cloudflare Pages em `https://intl-ai.illo.fyi/` quando as alterações são fundidas no ramo `main`.

O processo de implantação:

1. Deteta alterações no diretório `docs/`
2. Executa `pnpm docs:build` para gerar ficheiros estáticos
3. Implanta a saída para Cloudflare Pages

Não são necessários passos de implementação manual.

## Antes de enviar alterações na documentação

- Execute `pnpm docs:dev` e verifique se todas as páginas são renderizadas corretamente
- Execute `pnpm docs:validate` para verificar links quebrados
- Verificar que todos os exemplos de código estão completos e testados
- Siga as regras de estilo de escrita em `AGENTS.md` (sentence case, sem travessões, sem emoji)

## Código de conduta

Por favor, leia e siga o nosso [Code of Conduct](https://github.com/sigilco/intl-ai/blob/main/CODE_OF_CONDUCT.md).

## Perguntas ou questões?

- [Abrir uma questão](https://github.com/sigilco/intl-ai/issues) para erros de documentação ou sugestões
- [Start a discussion](https://github.com/sigilco/intl-ai/discussions) para perguntas
