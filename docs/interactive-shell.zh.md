# 交互式 Shell

语言: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

AgentHub 的默认体验是本地 chat shell：

```bash
agenthub
# 或
agenthub shell
```

Shell 会恢复最近的 chat，在紧凑 header 中显示当前 provider，然后让你直接输入普通任务。在没有 AgentHub project 的目录中，它会停留在 Chat Mode，并把 session state 存到 AgentHub user data directory，而不是创建 Git 或 `.agent`。Project bootstrap 会推迟到 file-changing transaction 真正需要时。你不需要先运行 `init`、`doctor`、`plan` 或 `run`。Built-in standard skills 已内置在 binary 中，因此 fresh project 进入 project mode 后可以立即使用 core file/page/Django workflows。

```text
agenthub> add a /courses page in the dashboard style
```

随后 AgentHub 会：

1. 如果有 `@` context，就加入 file、folder、transaction 或 memory context；
2. 将消息写入 chat transcript；
3. 创建 draft AgentSpec；
4. 显示 plan、provider、verifier、scope 和 commands；
5. 询问 inline approval；
6. 确认后运行 transaction；
7. 打印 diff、logs、report、explanation 和 undo 的 next actions。

## 输入模型

```text
普通文本          计划、确认、然后执行
/                 显示命令，并支持 tab completion
/cd ../other-app   不重启就切换到另一个 project folder
@README.md        给下一条请求附加 file context
@src              给下一条请求附加 folder summary
@last / @tx       附加 latest transaction summary
@tx:tx-123        附加指定 transaction summary
@memory:auth      附加相关 memory facts 和 warnings
!git status       通过 policy 检查运行 shell command 并记录日志
# use fetch only  保存 typed memory note
```

在 initialized projects 中，history 存在 `.agent/shell/history.txt`，chat transcripts 存在 `.agent/shell/chats/`。在没有 project bootstrap 的 Chat/Ops Mode 中，相同数据存到 AgentHub user data directory。

## Inline Approval

执行前，shell 会显示 plan、scope、commands、risk level、patch preview、verifier plan、rollback receipts 和 protected-path warnings。Approval prompt 支持：

```text
Enter/Y    approve once and run transaction
n/q        reject and keep draft
diff/x     show planned scope and diff preview before execution
r          show rollback receipts
v          show verifier plan
details/d  print full AgentSpec YAML
edit/e     open draft in $VISUAL or $EDITOR, then revalidate it
```

## 核心 Slash Commands

```text
/help             shell help
/cd <folder>      切换 working folder
/status           当前 project 和 transaction
/providers        provider wizard: status、roles、profiles 和 next actions
/memory           inspect memory
/skills           inspect skills
/transactions     recent transactions
/new              新 chat
/resume           resume selected/latest blocked transaction
/diff             diff selected/latest transaction
/logs             logs selected/latest transaction
/report           report selected/latest transaction
/explain          explain selected/latest transaction
/dashboard        打开 dashboard
/serve            启动 live local dashboard
/config           configuration
/clear            清空 terminal
/exit             退出
```

`/chats` 可以直接在 shell 中过滤：

```text
/chats status:COMMITTED
/chats provider:deepseek
/chats date:today
/chats status:BLOCKED_ON_HUMAN provider:kimi
```

`agenthub run`、`agenthub tx report`、`agenthub tx diff` 和 `agenthub tx logs` 等 expert commands 仍然可用于 scripts 和 CI。

## 边界

Shell uses AgentHub-owned DeepSeek/Kimi API providers for LLM work. It gives transaction control, approvals, logs, rollback, reports, memory, and dashboard visibility around provider work.
