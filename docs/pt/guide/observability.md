---
title: Observabilidade
description: Monitore o pipeline intl-ai. Relatórios JSON, tipos de descoberta, pontuações de qualidade e verificações exec como hooks personalizados.
---

# Observabilidade

O binário `intl-ai` emite saída legível por máquina em cada comando. Não há uma API de callback para configurar: pipe `--format json` para o que você usar para telemetria, ou adicione um `exec` check para instrumentação personalizada.

## Relatórios JSON

Cada comando aceita `--format json`:

```bash
intl-ai fill --format json | jq '{translated, written, unresolved: (.unresolved | length)}'
intl-ai check --format json | jq '.locales.es.invalid'
intl-ai status --format json
```

Erros em stderr têm o mesmo formato: `{"error": {"message": ..., "code": ...}}`.

## Descobertas

`intl-ai check` relata descobertas por chave. Tipos:

| Tipo         | Significado                                                                                  |
| ------------ | -------------------------------------------------------------------------------------------- |
| `missing`    | Chave ausente de um arquivo de localização alvo.                                             |
| `stale`      | Valor de propriedade de IA cuja fonte mudou desde o preenchimento.                           |
| `modified`   | Valor de propriedade humana cuja fonte mudou desde a revisão.                                |
| `unreviewed` | Chave nunca marcada como `reviewed` (ou re-sinalizada por `[quality]`).                      |
| `extra`      | Chave presente em um arquivo alvo, mas não na fonte.                                         |
| `invalid`    | Uma entrada `[[checks]]` rejeitou o valor (icu, parity, judge, spec, exec, ou `fail_below`). |

Falhas de nível de verificação (uma especificação que não carrega, um exec que não inicia, um erro de fornecedor) aparecem no array `errors` do relatório e sempre falham a execução. Reproduzir uma descoberta da cache incremental marca-a como `"cached": true` na saída JSON.

## Pontuações de qualidade no lockfile

`fill` regista o que produziu nos fragmentos `intl-ai.lock.d/`. Por chave:

- `origin`: `ai` ou `human`.
- `reviewed`: reposto para `false` automaticamente quando um humano edita o ficheiro.
- `quality.scores`: pontuações normalizadas por verificação a partir da última execução de preenchimento.
- `quality.unresolved`: chaves que falharam no gate `[fill].validate` e foram adotadas mesmo assim.

Rastreie a cobertura de revisão no CI com `intl-ai status --format json` e faça o merge das gates no `intl-ai check --fail-on`.

## Executa verificações como telemetria personalizada

Precisa de instrumentação por execução dentro do pipeline (postar no Sentry, emitir uma métrica, acrescentar a um log)? Escreva um `exec` check: o binário envia um pedido JSONL por lote de chaves e espera uma resposta JSONL com os resultados. É uma costura natural para "chamar isso em cada execução da verificação" sem fazer fork do pipeline.

```toml
[[checks]]
exec = "./scripts/i18n-telemetry"
timeout_ms = 60000
```

Veja `examples/checks/` para o protocolo.

## Códigos de saída

- `0`: sucesso (ou apenas descobertas abaixo dos tipos `--fail-on`).
- `1`: uma kind `--fail-on` foi encontrada, uma gate configurada falhou ou ocorreu um erro duro. No CI, não-zero sempre significa agir.

Veja [Configuração](/pt/guide/configuration/) for `[[checks]]`, `[check]`, `[fill]` e `[quality]`.
