# TUI Dashboard

Languages: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` renders a terminal dashboard for local AgentHub state. It is intentionally plain text, so it works in shells, CI logs, and remote terminals.

```bash
agenthub tui
```

Panels:

- `Transactions`: latest transaction ids and statuses from `.agent/tx`.
- `Latest Transaction`: DAG node/edge counts, DAG roles, verifier status, verifier log tail, cost, and estimated tokens.
- `Memory`: committed records, failed attempts, and recent workspace changes.
- `Approvals`: AgentSpec drafts with `approval_required: true` and transactions blocked on human input.

Example:

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
