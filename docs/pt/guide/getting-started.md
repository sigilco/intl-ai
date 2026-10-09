---
title: Início
description: Configure intl-ai em minutos. Instale o binário, configure o seu provedor e traduza.
---

# Início

intl-ai é um único binário que traduz os arquivos de localização que sua biblioteca i18n já lê. Este guia coloca‑o em funcionamento em minutos.

## Instalar o binário

Escolha o canal que corresponde ao seu fluxo de trabalho.

::: tabs

== tab "install script"

```sh
curl -fsSL https://intl-ai.illo.fyi/install.sh | bash
```

== tab "Homebrew"

```sh
brew install sigilco/tap-intl-ai/intl-ai
```

== tab "mise"

```sh
mise use npm:intl-ai@latest
```

== tab "npm"

```sh
npm install -g intl-ai
```

:::

No Windows, use o instalador do PowerShell:

```powershell
irm https://github.com/sigilco/intl-ai/releases/latest/download/intl-ai-installer.ps1 | iex
```

Ou execute‑o sem instalação global via `npx intl-ai` / `bunx intl-ai`. Verifique com `intl-ai --help`.

## 1. Inicializar

Execute `intl-ai init` no seu diretório raiz do projeto. Ele escreve `intl-ai.toml` e um fixture de replay `cassette.json` para que possa experimentar `fill` sem provedor configurado.

Apontar `locale_dir` para o diretório que sua biblioteca i18n lê, por exemplo `locales/`, `messages/` ou `public/locales/`.

## 2. Configure um provedor

`intl-ai.toml` necessita de um braço de provedor.

Ponto final HTTP compatível com OpenAI (qualquer servidor hospedado ou local que fala completas de chat):

```toml
[provider]
kind = "http"
model = "your-model"
api_key = "${env:OPENAI_API_KEY}"
# base_url = "https://api.openai.com/v1"  # optional override
```

Ou vá sem chave com o CLI do agente local:

```toml
[provider]
kind = "command"
agent = "claude-code" # claude-code | opencode | codex | crush | gemini
```

`api_key` aceita `${env:VAR}` e `${file:PATH}` interpolação, de modo que segredos nunca fiquem no ficheiro. Valide com `intl-ai config validate`.

## 3. Preencher traduções

```bash
intl-ai fill
```

Preenchimento é aditivo por padrão: escreve apenas as chaves ausentes e nunca altera valores existentes, quer sejam escritos por humanos ou gerados anteriormente. Os arquivos de localização alvo aparecem ao lado do seu código‑fonte, e `intl-ai.lock.d/` shards registram a origem de cada chave (`ai` ou `human`), o hash da fonte e o estado de revisão. Commit ambos.

::: warning Translations are ICU MessageFormat
intl-ai apenas emite e valida a sintaxe [ICU MessageFormat](https://unicode-org.github.io/icu/userguide/format_parse/messages/): placeholders `{name}`, `plural`/`select`/`selectordinal` com um braço `other` obrigatório, `#` pound. Esta é uma restrição rígida, não uma configuração: é o que permite que `check` e a validação em tempo de preenchimento detectem placeholders corrompidos. Sintaxe não-ICU (printf `%s`, mustache `{{x}}`) está fora de alcance.
:::

## 4. Verificar

```bash
intl-ai check
```

Check é somente leitura: relata chaves ausentes, desatualizadas, inválidas, modificadas, extras e não revisadas. Use `--fail-on missing,stale,invalid` em CI (código de saída 10 em achados) e `--format json` para saída legível por máquina.

## Integrações de ferramentas de build e frameworks

intl-ai combina com a biblioteca i18n que você já usa, em vez do seu bundler: aponte `locale_dir` para o diretório que a biblioteca lê e traduza no hook de ciclo de vida que sua pilha fornece. Para pilhas JS/TS, os shims nomeados fazem isso para você: `@intl-ai/unplugin` (Vite, Webpack, Rollup, esbuild, Rspack, Farm), `@intl-ai/next` para Next.js e `@intl-ai/expo` para React Native/Expo. Em outras pilhas, execute `intl-ai fill` diretamente (um script `prebuild` npm, uma fase de build Gradle ou Xcode, a ordem de `flutter gen-l10n`, e assim por diante).

Veja [sistemas de construção](/pt/guide/build-systems/) e as guias por biblioteca sob [bibliotecas i18n](/pt/guide/i18n-libraries/) para receitas concretas.
