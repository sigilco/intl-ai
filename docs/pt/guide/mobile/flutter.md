---
title: Flutter
description: Tradução de IA em tempo de compilação para i18n do Flutter. As traduções ocorrem durante a compilação, sem tempo de execução.
---

# Flutter

A integração do Flutter é fornecida como um construtor autônomo [`build_runner`](https://pub.dev/packages/build_runner) em [`examples/flutter/plugin`](https://github.com/sigilco/intl-ai/tree/main/examples/flutter/plugin). Ele executa `intl-ai fill` durante `flutter pub run build_runner build` e não tem sobrecarga de tempo de execução porque todas as traduções são escritas no disco antes que o aplicativo Flutter seja empacotado.

## Copiar o construtor

Copie `examples/flutter/plugin/` para o seu projeto Flutter (por exemplo, para `./tools/intl_ai_builder/`). O builder não é publicado como um pacote, portanto é seu e pode personalizar o código.

## Configurar `build.yaml`

Adicione o builder ao `build.yaml` da sua aplicação:

```yaml
targets:
  $default:
    builders:
      intl_ai_flutter|intl_ai:
        enabled: true
        options:
          configPath: "intl-ai.toml"
          executable: "intl-ai"
          verbose: false
```

Atualize o caminho `import` em `build.yaml` para corresponder ao local onde você copiou o construtor, por exemplo:

```yaml
builders:
  intl_ai:
    import: "package:my_app/tools/intl_ai_builder/lib/builder.dart"
```

## Criar `intl-ai.toml`

```toml
locale_dir = "assets/locales"
source = "en"
targets = ["es"]

[provider]
kind = "http"
base_url = "https://api.openai.com/v1"
model = "your-model-name"
api_key = "${env:OPENAI_API_KEY}"
```

## Executar build_runner

```bash
flutter pub get
flutter pub run build_runner build --delete-conflicting-outputs
```

O builder invoca:

```bash
intl-ai fill --config intl-ai.toml
```

## Opções do construtor

| Opção        | Tipo     | Predefinido    | Descrição                                                                                                  |
| ------------ | -------- | -------------- | ---------------------------------------------------------------------------------------------------------- |
| `configPath` | `String` | `intl-ai.toml` | Caminho para o seu ficheiro de configuração, relativo à raiz do pacote.                                    |
| `executable` | `String` | `intl-ai`      | Comando usado para invocar o CLI. Use `npx intl-ai` ou `bunx intl-ai` se não quiser uma instalação global. |
| `verbose`    | `bool`   | `false`        | Encaminhe a saída do CLI para o registo do build runner.                                                   |

## Como funciona

O construtor é registrado no ativo `$package$` e escreve um arquivo de marcador `intl-ai.done`. Quando o `build_runner` é executado, ele invoca a CLI `intl-ai` uma vez por pacote. Adicione `intl-ai.done` ao seu `.gitignore`; ele é usado apenas para rastrear a conclusão da compilação.

## Uso em tempo de execução

Carregue os arquivos JSON gerados com a sua biblioteca i18n preferida (`easy_localization`, `i18n_extension`, `flutter_gen`, etc.). O construtor apenas escreve traduções; ele não impõe uma API de tempo de execução.

## Requisitos

- Dart SDK 3.0+
- CLI `intl-ai` no `PATH`
- Um arquivo `intl-ai.toml` e um diretório de locale

## Exemplo

Consulte [`examples/flutter`](https://github.com/sigilco/intl-ai/tree/main/examples/flutter) para uma aplicação de trabalho completa.
