# TUI Dashboard

语言: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` 会渲染本地 AgentHub 状态的 terminal dashboard。它故意使用 plain text，因此可用于 shell、CI logs 和 remote terminals。

```bash
agenthub tui
```

面板：

- `Transactions`: 来自 `.agent/tx` 的最新 transaction ids 和 statuses。
- `Latest Transaction`: DAG node/edge counts、DAG roles、verifier status、verifier log tail、cost、estimated tokens。
- `Memory`: committed records、failed attempts、recent workspace changes。
- `Approvals`: 带 `approval_required: true` 的 AgentSpec drafts，以及等待 human input 的 transactions。

示例：

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
