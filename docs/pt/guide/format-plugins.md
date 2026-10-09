---
title: Plugins de formato
description: Ensine o intl-ai a ler e escrever qualquer formato de ficheiro de locale com um plugin externo.
---

# Plugins de formato

intl-ai inclui formatos de locale JSON e YAML incorporados. Uma entrada `[[formats]]` regista um formato personalizado suportado por um programa externo, para que qualquer formato de ficheiro (Android `strings.xml`, ARB, PO, `.strings`) possa funcionar com `intl-ai fill` e `intl-ai check` em qualquer linguagem de implementação, sem ligar Rust.

## Registrando um formato

```toml
format = "xml"

[[formats]]
name = "xml"
exec = "python3"
args = ["tools/strings_xml.py"]
extension = "xml"
```

| Chave                | Obrigatório | Propósito                                                                  |
| ------------------ | -------- | ------------------------------------------------------------------------ |
| `name`             | sim      | Nome do registo; `format = "<name>"` seleciona-o. Não deve ser `json`/`yaml` |
| `exec`             | sim      | Programa para iniciar (um processo por operação de leitura/escrita)                         |
| `args`             | não       | Argv passado para `exec`                                                    |
| `extension`        | sim      | Extensão canónica criada para novos ficheiros (`xml` -> `fr.xml`)             |
| `extensions`       | não       | Extensões extras que o formato reivindica no disco                               |
| `cwd`              | não       | Diretório de trabalho do plugin (padrão: o diretório do arquivo de configuração)          |
| `timeout_ms`       | não       | Orçamento em tempo real por operação (padrão `60000`)                               |
| `max_stdout_bytes` | não       | Limite do stdout em buffer (padrão 10 MiB; as respostas transportam ficheiros inteiros)        |

`format` aceita nomes integrados (`json`, `yaml`) ou um nome registado em `[[formats]]`. Um nome desconhecido falha a validação da configuração com a lista de nomes conhecidos.

::: warning
`[[formats]] exec` só pode aparecer no `intl-ai.toml` raiz, nunca num ficheiro incluído via `extends`. Esta é a mesma fronteira de confiança que `[[checks]] exec`: uma configuração base fornecida por uma dependência não deve registar programas para correr na sua máquina.
:::

## Como os arquivos são resolvidos

A resolução não muda: a extensão própria de um ficheiro existente prevalece, e o `format` configurado só cria novos ficheiros. Um projeto que mistura `en.json`, `fr.yaml` e `de.xml` num único `locale_dir` lê cada ficheiro com o formato que reivindicou a sua extensão.

## Protocolo v1

O plugin é uma transformação pura entre o texto do ficheiro e a árvore do corpus (um objeto JSON de string de chained aninhada -> string). O intl-ai é proprietário de toda a E/S de ficheiros: lê o ficheiro, passa o texto para o plugin e escreve o texto retornado de forma atómica. O plugin nunca toca no sistema de ficheiros.

Um processo filho por operação. stdin recebe exatamente uma linha de solicitação JSONL; stdout deve produzir exatamente uma linha de resposta JSONL. Ambos os lados são rigorosos sobre `v`: o intl-ai envia `v: 1` e exige `v: 1` de volta, e um plugin que recebe um `v` que não suporta deve responder com uma resposta `error`.

### Leitura

Requisição:

```json
{ "v": 1, "op": "read", "format": "xml", "path": "/abs/locales/fr.xml", "content": "<file text>" }
```

Resposta:

```json
{ "v": 1, "data": { "greeting": "Salut!", "nav": { "home": "Accueil" } } }
```

`data` deve ser um objeto JSON. `path` é informativo (use-o em mensagens de erro); plugins não devem abri-lo.

### Escrever

Requisição:

```json
{
  "v": 1,
  "op": "write",
  "format": "xml",
  "path": "/abs/locales/fr.xml",
  "data": { "greeting": "Salut!" }
}
```

Resposta:

```json
{ "v": 1, "content": "<resources>...</resources>\n" }
```

`content` é o texto completo do ficheiro; intl-ai escreve-o literalmente (ignorando a escrita quando os bytes não mudam).

### Erros e códigos de saída

Falhas ao nível do formato (entrada não parseável, construções não suportadas) são relatadas como uma resposta com um campo `error` e código de saída 0:

```json
{ "v": 1, "error": "parse:unclosed tag on line 3" }
```

`error` é texto livre; prefixá-lo com um código estável (`parse:`, `write:`) é a convenção. Uma saída não zero, um timeout ou uma linha de resposta mal formada ou ausente significa que o plugin despenhou e falha a operação da mesma forma, com o final do stderr anexado.

O texto do ficheiro é UTF-8; os formatos binários estão fora do âmbito da v1.

## Exemplo

[`examples/formats/strings-xml`](https://github.com/sigilco/intl-ai/tree/develop/examples/formats/strings-xml) é um plugin de referência: um único script de biblioteca padrão Python que lê e escreve um subconjunto mínimo de Android `strings.xml`. Como as chaves de strings.xml são planas, ele achata o corpus aninhado em nomes pontuados na escrita (`nav.home` -> `<string name="nav.home">`); um plugin pode, em vez disso, retornar objetos aninhados para preservar o agrupamento.

## Escrever o seu próprio

1. Ler uma linha JSON do stdin, escrever uma linha JSON no stdout.
2. Mantenha `v: 1` em cada resposta.
3. Nunca leia ou escreva `path`; transforme apenas `content`/`data`.
4. Relate falhas de formato como `{"v": 1, "error": "..."}` com saída 0; reserve saídas não-nulas para crashes.
5. Adicione a entrada ao `intl-ai.toml`, depois verifique com `intl-ai check` (lê todos os ficheiros de locale) e `intl-ai fill` (cria os que faltam).
