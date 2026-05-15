# AgentHub PRD v2

Languages: [English](prd-v2.en.md), [Русский](prd-v2.ru.md), [中文](prd-v2.zh.md), [Қазақша](prd-v2.kk.md)

AgentHub v2 moves the project from feature-complete foundation to hardened platform. The first priorities are effect-aware transactions, smart sync, real LLM gateway control, typed VCM-OS memory, hardened runners, then AAL v2 and team/hosted surfaces.

The working backlog is in `prd/v2/`. The effect ledger foundation is done: `.agent/tx/<tx-id>/effects.jsonl` records planned, applied, verified, rollback-pending, rolled-back, and non-rollbackable effects. The next task is concrete rollback handlers.
