# Local shell

Тілдер: [English](local-shell.en.md), [Русский](local-shell.ru.md), [中文](local-shell.zh.md), [Қазақша](local-shell.kk.md)

Іске қосу:

```bash
agenthub
```

Бұл daily work үшін recommended interface. AgentHub latest chat ашады, compact working-folder/provider header көрсетеді және task-ты бірден жазуға мүмкіндік береді. Initialized емес folder ішінде ол Chat Mode ішінде қалады және Git/`.agent` bootstrap-ты file-changing project transaction керек болғанға дейін кейінге қалдырады:

```text
agenthub> fix the failing runtime smoke test and keep files under 200 lines
```

Shell draft plan жасайды, scope, commands, patch preview, verifier plan, protected-path warnings және rollback receipts көрсетеді, approval сұрайды, transaction engine арқылы іске қосады және report, logs, diff, effects ledger, memory records және dashboard data қалдырады.

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
/memory inbox         review memory candidates
/ops                  host profiles, runbooks, receipts
/new                  new chat
/exit                 exit
@path                 attach file/folder context
@last                 attach latest report
!command              policy-checked shell command
# note                save memory
```

Ordinary text — негізгі жол. `ask`, `run`, `mode`, `watch`, `approve`, `resume`, `effects`, `memory`, `skills` және `undo` сияқты expert commands қажет кезде қолжетімді.

## Storage

- Shell history: initialized projects үшін `.agent/shell/history.txt`, Chat/Ops Mode үшін AgentHub user data directory
- Chats: initialized projects үшін `.agent/shell/chats/`, Chat/Ops Mode үшін AgentHub user data directory
- Memory: initialized projects үшін `.agent/memory/`, Chat/Ops Mode үшін AgentHub user data directory
- Ops state: host profiles, runbooks, and command receipts under the AgentHub user data directory
- Transactions: `.agent/tx/<tx-id>/`
- Dashboard: `.agent/reports/dashboard/index.html`

## Safety

Local shell `agenthub run` сияқты runtime қолданады: isolated workspace preparation, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion rules және reports.
