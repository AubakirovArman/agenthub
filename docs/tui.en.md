# TUI Dashboard

Languages: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` renders a terminal dashboard for local AgentHub state. It is intentionally plain text, so it works in shells, CI logs, and remote terminals.

```bash
agenthub tui
agenthub tui --live
```

Panels:

- `Summary`: total transactions and counts for committed, rolled back, blocked, and running states.
- `Transactions`: latest transaction ids and statuses from `.agent/tx`.
- `Latest Transaction`: current stage, last event, DAG node/edge counts, DAG roles, verifier status, verifier log tail, cost, estimated tokens, provider, effect count, heartbeat, and last output tail.
- `Memory`: committed records, failed attempts, and recent workspace changes.
- `Approvals`: AgentSpec drafts with `approval_required: true` and transactions blocked on human input.
- `Next Actions`: command suggestions for the latest or blocked transaction.

`--live` refreshes the same plain-text dashboard until interrupted. Use `--interval-ms <n>` to control refresh speed; `--once` renders one live frame for scripts and tests.

Example:

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
