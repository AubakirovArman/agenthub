# TUI Dashboard

Тілдер: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` жергілікті AgentHub күйін terminal dashboard ретінде көрсетеді. Формат plain text, сондықтан shell, CI logs және remote terminals ішінде жұмыс істейді.

```bash
agenthub tui
agenthub tui --live
```

Панельдер:

- `Summary`: transactions жалпы саны және committed, rolled back, blocked, running state counts.
- `Transactions`: `.agent/tx` ішіндегі соңғы transaction ids және statuses.
- `Latest Transaction`: ағымдағы stage, last event, DAG node/edge counts, DAG roles, verifier status, verifier log tail, cost, estimated tokens, provider, effects саны, heartbeat және last output tail.
- `Memory`: committed records, failed attempts және recent workspace changes.
- `Approvals`: `approval_required: true` бар AgentSpec drafts және human input күтіп тұрған transactions.
- `Next Actions`: latest немесе blocked transaction үшін command suggestions.

`--live` сол plain-text dashboard мәнін тоқтатылғанға дейін жаңартып тұрады. `--interval-ms <n>` refresh жиілігін басқарады; `--once` scripts және tests үшін бір live frame шығарады.

Мысал:

```text
AgentHub TUI Dashboard
Project: /repo

[Summary]
- total transactions: 1
- committed: 1 | rolled back: 0 | blocked: 0 | running: 0

[Transactions]
- tx-20260515030834-2aefeacd NOOP

[Latest Transaction]
- DAG: 5 nodes, 4 edges
- verifier passed: true
- cost: 0.000000 USD
- provider: codex
- effects: 4

[Next Actions]
- agenthub tx report tx-20260515030834-2aefeacd
```
