# TUI Dashboard

Языки: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` выводит terminal dashboard локального состояния AgentHub. Формат намеренно plain text, поэтому он работает в shell, CI logs и remote terminals.

```bash
agenthub tui
agenthub tui --live
```

Панели:

- `Summary`: общее число transactions и counts для committed, rolled back, blocked и running states.
- `Transactions`: последние transaction ids и statuses из `.agent/tx`.
- `Latest Transaction`: текущий stage, last event, DAG node/edge counts, DAG roles, verifier status, verifier log tail, cost, estimated tokens, provider, число effects, heartbeat и last output tail.
- `Memory`: committed records, failed attempts и recent workspace changes.
- `Approvals`: AgentSpec drafts с `approval_required: true` и transactions, ожидающие human input.
- `Next Actions`: command suggestions для latest или blocked transaction.

`--live` обновляет тот же plain-text dashboard до прерывания. `--interval-ms <n>` управляет частотой обновления; `--once` выводит один live frame для scripts и tests.

Пример:

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
