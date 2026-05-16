# AgentHub LLM Gateway

Languages: [English](llm-gateway.en.md), [Русский](llm-gateway.ru.md), [中文](llm-gateway.zh.md), [Қазақша](llm-gateway.kk.md)

## Purpose

The LLM Gateway is the provider control and observability boundary for model work. It records planned provider calls, prompt/context hashes, retry/failover metadata, budget decisions, redacted traces, optional raw traces, token estimates, and cost estimates.

## Transaction Artifacts

Every transaction now writes:

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

`context_pack.json` and `redacted_api.jsonl` are redacted by default. `redaction_report.json` records secret-like finding categories and counts without storing secret values.

## Provider Plan

`llm_provider_plan.json` normalizes AgentHub-owned API providers into one request model. Each planned call includes provider metadata, token counts, retry backoff, and explicit failover records when a requested adapter is routed to another provider.

Example:

```json
{
  "provider": { "id": "deepseek", "kind": "api_provider", "supports_streaming": true },
  "retry_policy": { "max_attempts": 3, "backoff_ms": [250, 1000, 3000] },
  "failover": []
}
```

## Real Provider Execution

v0.4 keeps planned metadata compatibility while moving user-facing provider execution to DeepSeek/Kimi APIs:

- `HttpProvider` can call DeepSeek/Kimi OpenAI-compatible `http://` or `https://` endpoints at `/v1/chat/completions`, with timeout, bearer token, and structured error body handling. It can also probe optional `/v1/models`; missing model-list support is reported without failing the completion test.
- `HttpProvider` can send OpenAI-compatible `tools` and `tool_choice`, preserve parsed non-streaming `message.tool_calls`, and expose them through the normalized `LlmResponse` for AgentHub-owned tool-loop receipts.
- `complete_with_retry` wraps provider calls with retry/backoff and optional attempt transcript records.
- Non-project chat turns use `provider.role.chat` and `provider.fallback.chat` to try multiple API providers inside AgentHub. Failed providers emit `provider_finished` with the error, `provider_fallback` names the next provider, and the turn emits one final `turn_finished` receipt.

Provider tests:

```bash
DEEPSEEK_API_KEY=... agenthub providers test deepseek
KIMI_API_KEY=... agenthub providers test kimi
```

Optional variables:

```text
DEEPSEEK_API_BASE_URL
DEEPSEEK_MODEL
KIMI_API_BASE_URL
KIMI_BASE_URL
MOONSHOT_API_BASE_URL
MOONSHOT_BASE_URL
KIMI_MODEL
KIMI_API_MODEL
AGENTHUB_KIMI_THINKING
KIMI_THINKING
```

For Kimi K2.6/K2.5 thinking-capable models, AgentHub sends `thinking: {"type":"disabled"}` by default to keep chat/project turns fast and cost-bounded. Set `AGENTHUB_KIMI_THINKING=enabled` when you explicitly want Kimi's thinking mode.

## Budget Policy

Set a transaction budget through `topology.routing.max_estimated_cost_usd`:

```yaml
topology:
  routing:
    max_estimated_cost_usd: 0.25
```

If planned model cost exceeds the limit, AgentHub writes `llm_budget.json` and blocks before execution.

## Raw Debug Mode

Raw context and raw API traces are written only when explicitly enabled:

```bash
AGENTHUB_RAW_TRACES=1 agenthub run examples/command-task.yaml
```

If the context scan finds secret-like values, raw context output is blocked even when `AGENTHUB_RAW_TRACES=1`. For a controlled local debug session you can override this with:

```bash
AGENTHUB_RAW_TRACES=1 AGENTHUB_ALLOW_RAW_SECRET_TRACES=1 agenthub run examples/command-task.yaml
```

Do not use that override in shared projects or CI.

That creates:

```text
.agent/tx/<tx-id>/raw_context_pack.json
.agent/tx/<tx-id>/raw_api.jsonl
```

## Cost Estimates

Local `command` adapter calls cost `0.0` by default. To configure a temporary estimate:

```bash
AGENTHUB_INPUT_USD_PER_1K=0.001 AGENTHUB_OUTPUT_USD_PER_1K=0.002 agenthub run examples/command-task.yaml
```

The estimate is stored in `cost.json` and summarized in `report.md`. Non-project API chat uses the same pricing table: `provider_finished` and `turn_finished` chat events include `estimated_input_cost_usd`, `estimated_output_cost_usd`, `estimated_cost_usd`, and `pricing_source`, and those fields are exposed by `agenthub exec --jsonl` and `/api/events`.
