---
title: Fornecedores
description: Referência do fornecedor intl-ai. Transportes HTTP (compatível com OpenAI), comandos (agentes CLI locais) e transportes de reprodução.
---

# Fornecedores

intl-ai traduz chaves de localidade chamando um modelo de IA. O bloco `[provider]` em `intl-ai.toml` seleciona o transporte; tudo o resto (contrato do prompt, agrupamento, tentativas, o juiz) é partilhado.

## `kind = "http"`

Um ponto final de conclusões de chat compatível com OpenAI. A superfície mais ampla: OpenAI, OpenRouter, LM Studio, Ollama, vLLM, Azure OpenAI e a maioria dos fornecedores alojados expõem esta forma.

```toml
[provider]
kind = "http"
provider = "openai"                 # wire-shape preset (default "openai")
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"

[provider.model_params]
temperature = 0.2                   # extra request-body params, spread last
```

| Campo          | Descrição                                                                                       |
| -------------- | ----------------------------------------------------------------------------------------------- |
| `provider`     | Nome predefinido da forma de transmissão; predefinição como `openai`.                           |
| `model`        | Identificador do modelo enviado no corpo da requisição.                                         |
| `api_key`      | Enviado como `Authorization: Bearer <key>`. Suporta interpolação `${env:VAR}` e `${file:PATH}`. |
| `base_url`     | Substituição de endpoint; padrão para o URL canónico do provedor.                               |
| `model_params` | Parâmetros extra do corpo da requisição, espalhados por último para que prevaleçam.             |

Os pedidos são repetidos com retardo exponencial até `max_retries` (padrão 3).

## `kind = "command"`

Executa um agente CLI local e lê as traduções de volta a partir do seu stdout. Utilize isto para fornecedores com uma forma de transmissão diferente, agentes CLI que já possui (um predefinição `agent` nomeia um conhecido), ou ferramentas personalizadas.

```toml
[provider]
kind = "command"
command = "my-translator"
args = ["--json"]
prompt_via = "stdin"        # or "argv" to append the prompt as the last arg
cwd = "."
timeout_ms = 300000
```

| Campo              | Descrição                                                                                 |
| ------------------ | ----------------------------------------------------------------------------------------- |
| `command`          | Linha de comando livre. Apenas configuração raiz, nunca via `extends`.                    |
| `agent`            | Nome predefinido da tabela interna de agentes (kebab-case).                               |
| `args`             | Argumentos extras passados ao comando.                                                    |
| `prompt_via`       | `stdin` (predefinição) ou `argv`.                                                         |
| `cwd`              | Diretório de trabalho, resolvido em relação ao diretório do arquivo de configuração.      |
| `timeout_ms`       | Orçamento de relógio de parede antes do SIGTERM, depois do SIGKILL (predefinição 300000). |
| `max_stdout_bytes` | Capacidade de stdout bufferizado.                                                         |

Porque `command` pode gerar programas arbitrários, é recusado em arquivos de configuração `extends`-ed: dados de uma dependência não devem executar código.

## `kind = "replay"`

Um cassino JSON de traduções predefinidas, para testes e demonstrações. Sem rede, sem credenciais, saída determinística.

```toml
[provider]
kind = "replay"
file = "cassette.json"
```

O cassete mapeia `{ "<locale>": { "<source value>": "<translation>" } }` e é resolvido em relação ao diretório do arquivo de configuração.

## O contrato do prompt

Cada transporte responde ao mesmo prompt: as chaves planas da localidade de origem mais instruções, termos do glossário e o contrato de marcadores de posição (`processor = "icu"` exige ICU MessageFormat). A resposta deve ser um objeto JSON de traduções. `fill` analisa-o rigorosamente; a verificação `judge` avalia a qualidade através do mesmo transporte.

Consulte [Configuração](/pt/guide/configuration/) para o esquema completo.
