# 本地 Shell

语言: [English](local-shell.en.md), [Русский](local-shell.ru.md), [中文](local-shell.zh.md), [Қазақша](local-shell.kk.md)

运行：

```bash
agenthub
```

这是推荐的日常入口。AgentHub 会打开 latest chat，在可能时准备项目，显示 readiness hints，然后让你直接输入任务：

```text
agenthub> fix the failing runtime smoke test and keep files under 200 lines
```

Shell 会创建 draft plan，显示将要运行的内容，询问 approval，通过 transaction engine 执行，并留下 report、logs、diff、effects ledger、memory records 和 dashboard data。

## 常用输入

```text
/help                 commands
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

普通文本是主路径。需要时仍可使用 expert commands：`ask`、`run`、`mode`、`watch`、`approve`、`resume`、`effects`、`memory`、`skills` 和 `undo`。

## Storage

- Shell history: `.agent/shell/history.txt`
- Chats: `.agent/shell/chats/`
- Transactions: `.agent/tx/<tx-id>/`
- Dashboard: `.agent/reports/dashboard/index.html`

## Safety

Local shell 使用与 `agenthub run` 相同的 runtime：isolated workspace preparation、command policy、bounded logs、verifier checks、diff guard、effect ledger、rollback、smart sync、memory promotion rules 和 reports。
