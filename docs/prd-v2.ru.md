# AgentHub PRD v2

Языки: [English](prd-v2.en.md), [Русский](prd-v2.ru.md), [中文](prd-v2.zh.md), [Қазақша](prd-v2.kk.md)

AgentHub v2 переводит проект из feature-complete foundation в hardened platform. Первые приоритеты: effect-aware transactions, smart sync, настоящий LLM Gateway control, typed VCM-OS memory, hardened runners, затем AAL v2 и team/hosted surfaces.

Рабочий backlog находится в `prd/v2/`. Effect ledger foundation сделан: `.agent/tx/<tx-id>/effects.jsonl` записывает planned, applied, verified, rollback-pending, rolled-back и non-rollbackable effects. Следующая task — concrete rollback handlers.
