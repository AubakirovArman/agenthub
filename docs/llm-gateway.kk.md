# AgentHub LLM Gateway

Тілдер: [English](llm-gateway.en.md), [Русский](llm-gateway.ru.md), [中文](llm-gateway.zh.md), [Қазақша](llm-gateway.kk.md)

## Мақсаты

LLM Gateway — model work үшін provider control және observability boundary. Ол planned provider calls, prompt/context hashes, retry/failover metadata, budget decisions, redacted traces, optional raw traces, token estimates және cost estimates жазады.

## Транзакция artifacts

Әр транзакция қазір жазады:

```text
.agent/tx/<tx-id>/context_pack.json
.agent/tx/<tx-id>/context_pack_trace.json
.agent/tx/<tx-id>/model_call_metadata.json
.agent/tx/<tx-id>/llm_provider_plan.json
.agent/tx/<tx-id>/llm_budget.json
.agent/tx/<tx-id>/llm_gateway_summary.json
.agent/tx/<tx-id>/redacted_api.jsonl
.agent/tx/<tx-id>/redaction_report.json
.agent/tx/<tx-id>/cost.json
```

`context_pack.json` және `redacted_api.jsonl` default бойынша redacted болады. `redaction_report.json` secret-like finding түрлері мен санын жазады, бірақ secret мәндерін сақтамайды.

## Provider Plan

`llm_provider_plan.json` CLI wrappers және болашақ API providers үшін бір request model береді. Әр planned call ішінде provider metadata, token counts, retry backoff және requested adapter басқа provider-ге routed болса explicit failover records болады.

Мысал:

```json
{
  "provider": { "id": "codex", "kind": "cli_wrapper", "supports_streaming": true },
  "retry_policy": { "max_attempts": 3, "backoff_ms": [250, 1000, 3000] },
  "failover": []
}
```

## Real Provider Execution

PRD v3 алғашқы real execution paths қосады және planned metadata compatibility сақтайды:

- `CliProvider` configured CLI command template іске қосады, prompt file жазады, stdout/stderr жинайды және provider transcript JSONL қосады.
- `HttpProvider` `/v1/chat/completions` үшін OpenAI-compatible `http://` немесе `https://` endpoint шақыра алады, timeout, bearer token және structured error body handling қолдайды.
- `complete_with_retry` provider calls үшін retry/backoff және optional attempt transcript records қосады.

Local OpenAI-compatible endpoint тексеру:

```bash
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=https://api.example.com agenthub providers diagnose openai-http
```

Optional variables:

```text
AGENTHUB_OPENAI_COMPAT_API_KEY
AGENTHUB_OPENAI_COMPAT_MODEL
```

## Budget Policy

Transaction budget `topology.routing.max_estimated_cost_usd` арқылы беріледі:

```yaml
topology:
  routing:
    max_estimated_cost_usd: 0.25
```

Егер planned model cost limit мәнінен асса, AgentHub `llm_budget.json` жазады және execution басталғанға дейін block жасайды.

## Raw debug mode

Raw context және raw API traces тек нақты қосылғанда жазылады:

```bash
AGENTHUB_RAW_TRACES=1 agenthub run examples/command-task.yaml
```

Егер context scan secret-like values тапса, `AGENTHUB_RAW_TRACES=1` берілсе де raw context жазылмайды. Бақыланатын local debug үшін мұны нақты override жасауға болады:

```bash
AGENTHUB_RAW_TRACES=1 AGENTHUB_ALLOW_RAW_SECRET_TRACES=1 agenthub run examples/command-task.yaml
```

Бұл override-ты shared projects немесе CI ішінде қолданба.

Ол мыналарды жасайды:

```text
.agent/tx/<tx-id>/raw_context_pack.json
.agent/tx/<tx-id>/raw_api.jsonl
```

## Cost estimates

Local `command` adapter default бойынша `0.0` тұрады. Уақытша estimate былай беріледі:

```bash
AGENTHUB_INPUT_USD_PER_1K=0.001 AGENTHUB_OUTPUT_USD_PER_1K=0.002 agenthub run examples/command-task.yaml
```

Estimate `cost.json` ішіне жазылады және `report.md` ішінде көрсетіледі.
