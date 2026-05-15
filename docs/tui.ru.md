# TUI Dashboard

Языки: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` выводит terminal dashboard локального состояния AgentHub. Формат намеренно plain text, поэтому он работает в shell, CI logs и remote terminals.

```bash
agenthub tui
```

Панели:

- `Transactions`: последние transaction ids и statuses из `.agent/tx`.
- `Latest Transaction`: DAG node/edge counts, DAG roles, verifier status, verifier log tail, cost и estimated tokens.
- `Memory`: committed records, failed attempts и recent workspace changes.
- `Approvals`: AgentSpec drafts с `approval_required: true` и transactions, ожидающие human input.

Пример:

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
