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

`llm_provider_plan.json` normalizes CLI wrappers and future API providers into one request model. Each planned call includes provider metadata, token counts, retry backoff, and explicit failover records when a requested adapter is routed to another provider.

Example:

```json
{
  "provider": { "id": "codex", "kind": "cli_wrapper", "supports_streaming": true },
  "retry_policy": { "max_attempts": 3, "backoff_ms": [250, 1000, 3000] },
  "failover": []
}
```

## Real Provider Execution

PRD v3 adds first real execution paths while keeping planned metadata compatibility:

- `CliProvider` can run a configured CLI command template, write a prompt file, capture stdout/stderr, and append provider transcript JSONL.
- `HttpProvider` can call an OpenAI-compatible `http://` or `https://` endpoint at `/v1/chat/completions`, with timeout, bearer token, and structured error body handling.
- `complete_with_retry` wraps provider calls with retry/backoff and optional attempt transcript records.

Local OpenAI-compatible endpoint test:

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

The estimate is stored in `cost.json` and summarized in `report.md`.
