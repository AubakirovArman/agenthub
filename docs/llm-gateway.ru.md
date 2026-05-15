# AgentHub LLM Gateway

Языки: [English](llm-gateway.en.md), [Русский](llm-gateway.ru.md), [中文](llm-gateway.zh.md), [Қазақша](llm-gateway.kk.md)

## Назначение

LLM Gateway — provider control и observability boundary для model work. Он записывает planned provider calls, prompt/context hashes, retry/failover metadata, budget decisions, redacted traces, optional raw traces, token estimates и cost estimates.

## Артефакты транзакции

Каждая транзакция теперь пишет:

```text
.agent/tx/<tx-id>/context_pack.json
.agent/tx/<tx-id>/context_pack_trace.json
.agent/tx/<tx-id>/model_call_metadata.json
.agent/tx/<tx-id>/llm_provider_plan.json
.agent/tx/<tx-id>/llm_budget.json
.agent/tx/<tx-id>/llm_gateway_summary.json
.agent/tx/<tx-id>/redacted_api.jsonl
.agent/tx/<tx-id>/cost.json
```

`context_pack.json` и `redacted_api.jsonl` по умолчанию проходят redaction.

## Provider Plan

`llm_provider_plan.json` приводит CLI wrappers и будущие API providers к одной request model. Каждый planned call содержит provider metadata, token counts, retry backoff и explicit failover records, если requested adapter был routed к другому provider.

Пример:

```json
{
  "provider": { "id": "codex", "kind": "cli_wrapper", "supports_streaming": true },
  "retry_policy": { "max_attempts": 3, "backoff_ms": [250, 1000, 3000] },
  "failover": []
}
```

## Budget Policy

Transaction budget задаётся через `topology.routing.max_estimated_cost_usd`:

```yaml
topology:
  routing:
    max_estimated_cost_usd: 0.25
```

Если planned model cost превышает limit, AgentHub пишет `llm_budget.json` и блокирует запуск до execution.

## Raw debug mode

Raw context и raw API traces пишутся только при явном включении:

```bash
AGENTHUB_RAW_TRACES=1 agenthub run examples/command-task.yaml
```

Это создаёт:

```text
.agent/tx/<tx-id>/raw_context_pack.json
.agent/tx/<tx-id>/raw_api.jsonl
```

## Cost estimates

Local `command` adapter по умолчанию стоит `0.0`. Временную оценку можно задать так:

```bash
AGENTHUB_INPUT_USD_PER_1K=0.001 AGENTHUB_OUTPUT_USD_PER_1K=0.002 agenthub run examples/command-task.yaml
```

Оценка сохраняется в `cost.json` и показывается в `report.md`.
