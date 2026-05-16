# Local shell

Тілдер: [English](local-shell.en.md), [Русский](local-shell.ru.md), [中文](local-shell.zh.md), [Қазақша](local-shell.kk.md)

Іске қосу:

```bash
agenthub
```

Бұл daily work үшін recommended interface. AgentHub latest chat ашады, мүмкін болса project дайындайды, compact working-folder/provider header көрсетеді және task-ты бірден жазуға мүмкіндік береді:

```text
agenthub> fix the failing runtime smoke test and keep files under 200 lines
```

Shell draft plan жасайды, не орындалатынын көрсетеді, approval сұрайды, transaction engine арқылы іске қосады және report, logs, diff, effects ledger, memory records және dashboard data қалдырады.

## Useful inputs

```text
/help                 commands
/cd <folder>          project folder-ды restart жасамай ауыстыру
/status               current project, provider, transaction
/providers            setup and provider health
/transactions         recent transactions
/diff [tx]            transaction diff
/logs [tx|stage]      transaction logs
/report [tx]          report
/explain [tx]         result explanation
/serve [addr]         local live dashboard
/new                  new chat
/exit                 exit
@path                 attach file/folder context
@last                 attach latest report
!command              policy-checked shell command
# note                save project memory
```

Ordinary text — негізгі жол. `ask`, `run`, `mode`, `watch`, `approve`, `resume`, `effects`, `memory`, `skills` және `undo` сияқты expert commands қажет кезде қолжетімді.

## Storage

- Shell history: `.agent/shell/history.txt`
- Chats: `.agent/shell/chats/`
- Transactions: `.agent/tx/<tx-id>/`
- Dashboard: `.agent/reports/dashboard/index.html`

## Safety

Local shell `agenthub run` сияқты runtime қолданады: isolated workspace preparation, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion rules және reports.
