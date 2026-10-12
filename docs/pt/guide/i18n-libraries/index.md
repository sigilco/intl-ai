---
title: bibliotecas i18n
description: O intl-ai gera ficheiros de locale em tempo de compilação. Utilize com vue-i18n, i18next ou qualquer biblioteca i18n.
---

# bibliotecas i18n

O intl-ai gera ficheiros de tradução em tempo de construção. Escolhes a biblioteca i18n de runtime. Esta página lista as opções suportadas e a sua compatibilidade com ICU.

## ICU MessageFormat

A maioria das bibliotecas i18n modernas falam ICU MessageFormat. Defina `processor = "icu"` no `intl-ai.toml` para que o modelo emita sintaxe ICU; é o contrato que a verificação `icu` valida.

A principal exceção é o i18next, que utiliza a interpolação `{{var}}` e sufixos plurais em vez de sintaxe ICU.

## Compatibilidade de bibliotecas

| Biblioteca                    | ICU nativo                         | Configuração                    | Melhor para                                           |
| ----------------------------- | ---------------------------------- | ------------------------------- | ----------------------------------------------------- |
| react-intl                    | Sim                                | Nenhum                          | Aplicativos React que precisam de ICU                 |
| @formatjs/intl                | Sim                                | Nenhum                          | Runtime ICU independente de framework                 |
| lingui                        | Sim                                | Macro de build                  | Segurança em tempo de compilação, qualquer framework  |
| i18next-icu                   | Sim (via plugin)                   | Instale `i18next-icu`           | Aplicativos i18next existentes em migração para o ICU |
| vue-i18n + intl-messageformat | Sim (via compilador personalizado) | Personalizado `messageCompiler` | Aplicações Vue que precisam de ICU                    |
| svelte-i18n                   | Parcial                            | Chamadas manuais de formatação  | Aplicações Svelte                                     |
| @cookbook/solid-intl          | Sim                                | Nenhum                          | Apps Solid que precisam de ICU                        |
| @lit/localize                 | Parcial                            | Workaround da comunidade        | Componentes Web                                       |
| typesafe-i18n                 | Não                                | N/A                             | i18n type-safe sem ICU                                |
| rosetta                       | Não                                | N/A                             | Pegada mínima (298 bytes)                             |
| @solid-primitives/i18n        | Não                                | N/A                             | i18n Solid mínimo                                     |

## Configuração

Defina `processor = "icu"` no `intl-ai.toml` quando a sua biblioteca de destino utiliza ICU MessageFormat. A maioria das bibliotecas modernas faz.

Se utiliza o i18next, omita a definição do processador (o passthrough preserva o seu estilo `{{var}}`).

```toml
# intl-ai.toml
locale_dir = "locales"
source = "en"
targets = ["es", "fr"]
processor = "icu"   # recommended for most libraries
```

## Guias por biblioteca

- [Vue (vue-i18n)](/pt/guide/i18n-libraries/vue-i18n)
- [i18next](/pt/guide/i18n-libraries/i18next)

## Escolha de biblioteca

| Necessidade                               | Utilizar                                         |
| ----------------------------------------- | ------------------------------------------------ |
| React + ICU                               | `react-intl`                                     |
| Vue + ICU                                 | `vue-i18n` com o compilador `intl-messageformat` |
| Sintaxe i18next (sem ICU)                 | `i18next` + `react-i18next`                      |
| Segurança de tipos em tempo de compilação | `lingui` ou `typesafe-i18n`                      |
| Bundle mais pequeno                       | `rosetta`                                        |
