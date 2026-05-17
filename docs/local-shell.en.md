# Local Shell

Languages: [English](local-shell.en.md), [Русский](local-shell.ru.md), [中文](local-shell.zh.md), [Қазақша](local-shell.kk.md)

Run:

```bash
agenthub
```

This is the recommended daily interface. AgentHub opens the latest chat, shows a compact working-folder/provider header, and lets you type the task directly. In an uninitialized folder it stays in Chat Mode and defers Git/`.agent` bootstrap until a file-changing project transaction needs it:

```text
agenthub> fix the failing runtime smoke test and keep files under 200 lines
```

The shell creates a draft plan, shows scope, commands, patch preview, verifier plan, protected-path warnings, and rollback receipts, asks for approval, executes through the transaction engine, and leaves a report, logs, diff, effects ledger, memory records, and dashboard data.

## Useful Inputs

```text
/help                 commands
/cd <folder>          switch project folder without restarting
/status               current project, provider, transaction
/providers            setup and provider health
/transactions         recent transactions
/diff [tx]            transaction diff
/logs [tx|stage]      transaction logs
/report [tx]          report
/explain [tx]         result explanation
/serve [addr]         local live dashboard
/memory inbox         review memory candidates
/ops                  host profiles, runbooks, receipts
/new                  new chat
/exit                 exit
@path                 attach file/folder context
@last                 attach latest report
!command              policy-checked shell command
# note                save memory
```

Plain text is the main path. Expert commands like `ask`, `run`, `mode`, `watch`, `approve`, `resume`, `effects`, `memory`, `skills`, and `undo` are still available when needed.

## Storage

- Shell history: `.agent/shell/history.txt` for initialized projects, or the AgentHub user data directory for Chat/Ops Mode
- Chats: `.agent/shell/chats/` for initialized projects, or the AgentHub user data directory for Chat/Ops Mode
- Memory: `.agent/memory/` for initialized projects, or the AgentHub user data directory for Chat/Ops Mode
- Ops state: host profiles, runbooks, and command receipts under the AgentHub user data directory
- Transactions: `.agent/tx/<tx-id>/`
- Dashboard: `.agent/reports/dashboard/index.html`

## Safety

The local shell uses the same runtime as `agenthub run`: isolated workspace preparation, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion rules, and reports.
