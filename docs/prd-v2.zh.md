# AgentHub PRD v2

语言: [English](prd-v2.en.md), [Русский](prd-v2.ru.md), [中文](prd-v2.zh.md), [Қазақша](prd-v2.kk.md)

AgentHub v2 将项目从 feature-complete foundation 推进到 hardened platform。优先级依次是 effect-aware transactions、smart sync、真正的 LLM Gateway control、typed VCM-OS memory、hardened runners，然后是 AAL v2 和 team/hosted surfaces。

工作 backlog 位于 `prd/v2/`。effect ledger foundation 已完成：`.agent/tx/<tx-id>/effects.jsonl` 记录 planned、applied、verified、rollback-pending、rolled-back 和 non-rollbackable effects。下一项 task 是 concrete rollback handlers。
