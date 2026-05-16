# Локальный shell

Языки: [English](local-shell.en.md), [Русский](local-shell.ru.md), [中文](local-shell.zh.md), [Қазақша](local-shell.kk.md)

Запуск:

```bash
agenthub
```

Это рекомендованный ежедневный интерфейс. AgentHub открывает latest chat, показывает компактный header с working folder/provider и позволяет сразу писать задачу. В неинициализированной папке он остаётся в Chat Mode и откладывает Git/`.agent` bootstrap до file-changing project transaction:

```text
agenthub> fix the failing runtime smoke test and keep files under 200 lines
```

Shell создаёт draft plan, показывает что будет выполнено, спрашивает approval, запускает transaction engine и оставляет report, logs, diff, effects ledger, memory records и dashboard data.

## Полезный ввод

```text
/help                 commands
/cd <folder>          сменить project folder без перезапуска
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
# note                save memory
```

Обычный текст — главный путь. Expert commands `ask`, `run`, `mode`, `watch`, `approve`, `resume`, `effects`, `memory`, `skills` и `undo` остаются доступны, когда нужны.

## Storage

- Shell history: `.agent/shell/history.txt` для initialized projects или AgentHub user data directory для Chat/Ops Mode
- Chats: `.agent/shell/chats/` для initialized projects или AgentHub user data directory для Chat/Ops Mode
- Memory: `.agent/memory/` для initialized projects или AgentHub user data directory для Chat/Ops Mode
- Transactions: `.agent/tx/<tx-id>/`
- Dashboard: `.agent/reports/dashboard/index.html`

## Safety

Local shell использует тот же runtime, что и `agenthub run`: isolated workspace preparation, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion rules и reports.
