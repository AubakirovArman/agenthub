# TUI Dashboard

Languages: [English](tui.en.md), [Русский](tui.ru.md), [中文](tui.zh.md), [Қазақша](tui.kk.md)

`agenthub tui` renders a terminal dashboard for local AgentHub state. It is intentionally plain text, so it works in shells, CI logs, and remote terminals. The chat-facing panels are built from the same event store used by `agenthub exec --jsonl`; the TUI does not run a separate agent runtime.

```bash
agenthub tui
agenthub tui --live
```

Panels:

- `Status Line`: Chat/Ops/Project mode, active provider/model, Git/runtime state, latest chat, token/cost receipt, and controls such as `Ctrl-C`, `/resume`, `/messages`, and `/context`.
- `Composer`: input hint, slash palette, and `@` context mention forms.
- `Chat Transcript`: recent user, assistant, streaming, and tool transcript lines from the latest chat.
- `Event Rail`: recent `intent_classified`, `context_built`, `provider_requested`, `assistant_delta`, `tool_permission`, fallback, provider-finished, and turn-finished events with running/streaming/approval/error/done states.
- `Live Tool Cards`: chat tool permissions, approval-required stops, memory extraction, turn cost/tokens, native command-plan receipts, builtin tool-result reinjection receipts, policy summaries, and artifact links.
- `Summary`: total transactions and counts for committed, rolled back, blocked, and running states.
- `Transactions`: latest transaction ids and statuses from `.agent/tx`.
- `Latest Transaction`: current stage, last event, DAG node/edge counts, DAG roles, verifier status, verifier log tail, cost, estimated tokens, provider, effect count, heartbeat, and last output tail.
- `Providers`: default provider, ready/missing counts, named profile count, provider status lines, role assignments, and fallback chains.
- `Memory`: committed records, failed attempts, and recent workspace changes.
- `Approvals`: AgentSpec drafts with `approval_required: true` and transactions blocked on human input.
- `Next Actions`: command suggestions for the latest or blocked transaction.

`--live` refreshes the same plain-text dashboard until interrupted. Use `--interval-ms <n>` to control refresh speed; `--once` renders one live frame for scripts and tests.

Example:

```text
AgentHub TUI Dashboard
Project: /repo
Tabs: Chat | Events | Run | Transactions | Diff | Logs | Effects | Approvals | Memory | Providers

[Status Line]
- mode: project | provider: deepseek ok model:deepseek-chat | git ok | project runtime
- chat: chat-demo check server load
- tokens: prompt 64 total 69 | cost: 0.000010 USD
- controls: Ctrl-C interrupt | /resume | /messages | /context

[Composer]
- prompt: Type a request, / command, @ context, ! shell command, or # memory note
- slash palette:
  - /messages    show current chat transcript
- context mentions: @file @folder @tx:latest @chat:latest @memory:summary

[Live Tool Cards]
- [memory] memory: memory extraction
  memory extraction added 2 inbox candidate(s)
- [done] command_plan: tx-20260515030834-2aefeacd executor command plan
  status ok source native_tool_call native_calls 1 commands 1 approvals 0
- [done] tool_results: tx-20260515030834-2aefeacd executor tool results
  status ok rounds 1 results 1 approvals 0 protected 0 truncated 0 network_denied 0

[Summary]
- total transactions: 1
- committed: 1 | rolled back: 0 | blocked: 0 | running: 0

[Transactions]
- tx-20260515030834-2aefeacd NOOP

[Latest Transaction]
- DAG: 5 nodes, 4 edges
- verifier passed: true
- cost: 0.000000 USD
- provider: deepseek
- effects: 4

[Providers]
- default: deepseek
- ready: 1 | missing: 2 | profiles: 0
- executor -> deepseek (ok)
- reviewer -> kimi (missing) fallback:kimi,command

[Next Actions]
- agenthub tx report tx-20260515030834-2aefeacd
```
