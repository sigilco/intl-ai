---
title: Configuração do modelo de IA
description: Configura qualquer modelo de IA para intl-ai. Qualquer endpoint compatível com OpenAI, um agente CLI local ou uma cassete de reprodução.
---

# Configuração do modelo de IA

A tradução é estruturada e segue instruções. Os modelos económicos tratam-na bem, portanto não precisas de um modelo topo de gama.

## Escolhe um fornecedor

Três caminhos funcionam: um servidor local, um endpoint na cloud ou um agregador. Todos os três utilizam `kind = "http"` face a uma API de chat-completions compatível com OpenAI.

### Local: LM Studio ou Ollama

[LM Studio](https://lmstudio.ai) e [Ollama](https://ollama.com) executam modelos localmente e expõem um servidor compatível com OpenAI. Isto é ideal para desenvolvimento, teste e trabalho sensível à privacidade.

```toml
[provider]
kind = "http"
base_url = "http://127.0.0.1:1234/v1"
model = "your-local-model"
api_key = "lm-studio"
```

### Cloud: fornecedores compatíveis com OpenAI

Qualquer fornecedor com um endpoint compatível com OpenAI funciona da mesma forma: defina `base_url`, `model` e `api_key`.

```toml
[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

Anthropic, Google, Azure OpenAI, Cohere, Mistral e outros expõem superfícies compatíveis com OpenAI (de primeira parte ou através de um proxy). Aponte `base_url` para o teu e defina `model` conforme necessário. Para uma API com uma forma de fio diferente, utilize o fornecedor `command` para executar um agente local em vez disso; veja [Providers](/pt/guide/providers/).

### Agregador: OpenRouter

[OpenRouter](https://openrouter.ai) fornece uma única chave de API para vários fornecedores e um nível gratuito.
Um modelo gratuito estável no momento da escrita é `google/gemini-2.0-flash-exp:free`.
Se deixar de funcionar, verifica a lista de modelos gratuitos da OpenRouter e atualiza esta única referência.

```toml
[provider]
kind = "http"
base_url = "https://openrouter.ai/api/v1"
model = "google/gemini-2.0-flash-exp:free"
api_key = "${env:OPENROUTER_API_KEY}"
```

## Ajuste da solicitação

`model_params` espalha campos extra no corpo do pedido por último, assim os teus parâmetros têm precedência sobre os predefinidos:

```toml
[provider.model_params]
temperature = 0.2
max_tokens = 1024
```

## Janela de contexto

Utilize um modelo com uma janela de contexto mínima de 16,000 tokens. Todos os fornecedores listados acima excedem este valor.

## Resolução de problemas

Se a tradução falhar, verifica:

- A interpolação de `api_key` é resolvida (`intl-ai config validate` mostra-a mascarada).
- O servidor do fornecedor é acessível a partir da tua máquina (`base_url`).
- O teu identificador de `model` corresponde à documentação do fornecedor.
