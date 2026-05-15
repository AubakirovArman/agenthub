# TUI Dashboard

Тілдер: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` жергілікті AgentHub күйін terminal dashboard ретінде көрсетеді. Формат plain text, сондықтан shell, CI logs және remote terminals ішінде жұмыс істейді.

```bash
agenthub tui
```

Панельдер:

- `Transactions`: `.agent/tx` ішіндегі соңғы transaction ids және statuses.
- `Latest Transaction`: DAG node/edge counts, DAG roles, verifier status, verifier log tail, cost және estimated tokens.
- `Memory`: committed records, failed attempts және recent workspace changes.
- `Approvals`: `approval_required: true` бар AgentSpec drafts және human input күтіп тұрған transactions.

Мысал:

```text
AgentHub TUI Dashboard
Project: /repo

[Transactions]
- tx-20260515030834-2aefeacd NOOP

[Latest Transaction]
- DAG: 5 nodes, 4 edges
- verifier passed: true
- cost: 0.000000 USD
```
