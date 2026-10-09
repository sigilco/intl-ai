---
title: Comenzar
description: Configura intl-ai en minutos. Instala el binario, configura tu proveedor y traduce.
---

# Comenzar

intl-ai es un solo binario que traduce los archivos de localización que tu biblioteca i18n ya lee. Esta guía lo pone en marcha en minutos.

## Instalar el binario

Elige el canal que coincida con tu flujo de trabajo.

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

En Windows, usa el instalador de PowerShell:

```powershell
irm https://github.com/sigilco/intl-ai/releases/latest/download/intl-ai-installer.ps1 | iex
```

O bien ejecútalo sin instalación global mediante `npx intl-ai` / `bunx intl-ai`. Verifica con `intl-ai --help`.

## 1. Inicializar

Ejecuta `intl-ai init` en la raíz de tu proyecto. Escribe `intl-ai.toml` y un fixture de replay `cassette.json` para que puedas probar `fill` sin proveedor configurado.

Apunta `locale_dir` al directorio que tu biblioteca i18n lee, por ejemplo `locales/`, `messages/` o `public/locales/`.

## 2. Configura un proveedor

`intl-ai.toml` necesita un brazo de proveedor.

Punto final HTTP compatible con OpenAI (cualquier servidor alojado o local que habla completaciones de chat):

```toml
[provider]
kind = "http"
model = "your-model"
api_key = "${env:OPENAI_API_KEY}"
# base_url = "https://api.openai.com/v1"  # optional override
```

O bien ve sin clave con la CLI del agente local:

```toml
[provider]
kind = "command"
agent = "claude-code" # claude-code | opencode | codex | crush | gemini
```

`api_key` acepta la interpolación `${env:VAR}` y `${file:PATH}`, de modo que los secretos nunca estén en el archivo. Valida con `intl-ai config validate`.

## 3. Rellenar traducciones

```bash
intl-ai fill
```

Fill es aditivo por defecto: escribe solo las claves faltantes y nunca toca los valores existentes, ya sean escritos a mano o generados previamente. Los archivos de localización objetivo aparecen junto a tu fuente, y `intl-ai.lock.d/` registra el origen de cada clave (`ai` o `human`), el hash de origen y el estado de revisión. Commitea ambos.

::: warning Translations are ICU MessageFormat
intl-ai solo emite y valida [ICU MessageFormat](https://unicode-org.github.io/icu/userguide/format_parse/messages/) la sintaxis: `{name}` placeholders, `plural`/`select`/`selectordinal` con un brazo `other` requerido, `#` pound. Esta es una restricción dura, no un ajuste: es lo que permite que `check` y la compuerta de validación en tiempo de relleno detecten placeholders mangled. La sintaxis no ICU (printf `%s`, mustache `{{x}}`) está fuera de alcance.
:::

## 4. Verificar

```bash
intl-ai check
```

Check es de solo lectura: informa sobre claves faltantes, obsoletas, inválidas, modificadas, extra y sin revisar. Usa `--fail-on missing,stale,invalid` en CI (código de salida 10 en hallazgos) y `--format json` para salida legible por máquina.

## Integraciones de herramientas y frameworks de construcción

intl-ai se asocia con la biblioteca i18n que ya usas en lugar de tu bundler: apunta `locale_dir` al directorio que lee la biblioteca, luego traduce en el gancho de ciclo de vida que tu stack proporciona. Para stacks JS/TS los shims asignados hacen esto por ti: `@intl-ai/unplugin` (Vite, Webpack, Rollup, esbuild, Rspack, Farm), `@intl-ai/next` para Next.js, y `@intl-ai/expo` para React Native/Expo. En otros stacks, ejecuta directamente `intl-ai fill` (un script `prebuild` de npm, una fase de compilación Gradle o Xcode, ordenación `flutter gen-l10n`, y así sucesivamente).

Ve a [Build systems](/es/guide/build-systems/) y las guías por biblioteca bajo [i18n libraries](/es/guide/i18n-libraries/) para recetas concretas.
