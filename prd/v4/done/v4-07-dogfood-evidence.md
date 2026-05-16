# V4.07 Dogfood Evidence

## Status

Done.

## Completed

- Added `scripts/provider-dogfood.sh` for explicit live provider dogfooding.
- Wired optional provider dogfood into `scripts/dogfood.sh` through `AGENTHUB_DOGFOOD_PROVIDER`.
- Provider dogfood requires `AGENTHUB_PROVIDER_DOGFOOD_LIVE=1` before invoking a real model-backed CLI.
- Provider dogfood writes `target/dogfood/provider-dogfood-report.json` with provider, transaction id, status, report path, artifact directory, and token-observation note.
- The script verifies provider diagnostics, provider test output, adapter invocation artifacts, transaction report existence, no-commit behavior, and clean main state.
- Provider dogfood now persists `report.md`, `adapter_invocation_executor.json`, `agent_trace.json`, `effects.jsonl`, `journal.jsonl`, `llm_budget.json`, and `llm_provider_plan.json` into the provider artifact directory before the temporary project is cleaned up.
- Dogfooding docs were updated in English, Russian, Chinese, and Kazakh.

## Verified Evidence

- `AGENTHUB_DOGFOOD_STRESS_COUNT=100 scripts/dogfood.sh` passed on 2026-05-16.
- Stress report: `target/dogfood/dogfood-report.json`.
- Stress result: 100 requested, 100 completed, transaction SQLite index present, duration 104 seconds.
- `AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=codex AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh` passed on 2026-05-16.
- Live provider transaction: `tx-20260516061053-aa4996e8`.
- Live provider status: `NOOP`, no commit, verifier passed, main stayed clean.
- Persisted live provider report: `target/dogfood/provider-codex/report.md`.
- Persisted live provider evidence: `target/dogfood/provider-codex/adapter_invocation_executor.json`, `agent_trace.json`, `effects.jsonl`, `journal.jsonl`, `llm_budget.json`, and `llm_provider_plan.json`.

## 1.0 Relevance

This creates repeatable evidence for local long-running dogfood and real provider adapter smoke tests without accidentally spending provider tokens during normal CI or release-readiness runs.
