# TUI Dashboard

语言: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` 会渲染本地 AgentHub 状态的 terminal dashboard。它故意使用 plain text，因此可用于 shell、CI logs 和 remote terminals。

```bash
agenthub tui
agenthub tui --live
```

面板：

- `Summary`: transactions 总数，以及 committed、rolled back、blocked、running 状态计数。
- `Transactions`: 来自 `.agent/tx` 的最新 transaction ids 和 statuses。
- `Latest Transaction`: 当前 stage、last event、DAG node/edge counts、DAG roles、verifier status、verifier log tail、cost、estimated tokens、provider、effects count、heartbeat 和 last output tail。
- `Memory`: committed records、failed attempts、recent workspace changes。
- `Approvals`: 带 `approval_required: true` 的 AgentSpec drafts，以及等待 human input 的 transactions。
- `Next Actions`: 针对 latest 或 blocked transaction 的 command suggestions。

`--live` 会持续刷新同一个 plain-text dashboard，直到用户中断。`--interval-ms <n>` 控制刷新频率；`--once` 为 scripts 和 tests 输出一个 live frame。

示例：

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
