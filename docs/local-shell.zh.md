# 本地 Shell

AgentHub 可以作为交互式本地 shell 运行：

```bash
agenthub
# 或
agenthub shell
```

这个 shell 面向 local-first 使用。你可以查看历史事务会话、打开报告、用自然语言生成 draft AgentSpec、在同一个 prompt 中运行请求，并保持一个当前选中的 transaction。这些会话是 AgentHub transaction sessions，不是自由聊天房间：每条被执行的消息都会成为可追踪的 transaction，并带有 report、journal、effects、verifier output 和 memory behavior。

Shell 默认以 `plan` 模式启动。在该模式下，普通文本只会创建 draft。使用 `mode run` 后，普通文本会立即执行。

## 命令

```text
help                         显示命令
init                         初始化 .agent
mode plan|run                设置普通文本行为
current                      显示当前选中的 transaction
close                        清除当前选中的 transaction
sessions or history          列出最近事务
session [tx-id|latest]       列出会话或打开一个会话
doctor                       检查本地 readiness
providers [status|...]       列出、setup、test 或 diagnose providers
provider <id>                设置 default provider
config [show|set key value]  查看或更新 config
dashboard                    写入本地 web dashboard
open <tx-id|latest>          打开事务报告并设为当前事务
latest                       打开最新 transaction
watch [tx-id|latest]         跟随实时 transaction journal
cancel [tx-id|latest]        请求取消 transaction
report [tx-id]               打印报告，默认使用当前事务
effects [tx-id]              打印 effect ledger
explain [tx-id]              解释结果、失败原因和下一步
memory [summary|audit]       显示 memory summary 或 audit
skills [scorecard]           列出 skills 或显示 scorecard
undo [tx-id|last]            git revert 一个 committed transaction
ask <request>                写入 draft AgentSpec
do <request>                 写入 draft 并立即执行
run <spec|request> [--no-commit]
quit                         退出
普通文本                     plan 模式: draft；run 模式: 执行
/sessions /open /report      交互式 slash 别名
```

## 示例

从消息创建 draft：

```text
agenthub> add /courses page in the dashboard style
draft .agent/drafts/shell-20260515123000.yaml
```

切换到立即执行：

```text
agenthub:plan> mode run
mode run
agenthub:run> add a generated health-check file
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

运行 spec：

```text
agenthub:plan> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

立即运行自然语言请求：

```text
agenthub:plan> do add a generated health-check file
```

浏览历史会话：

```text
agenthub:plan> sessions
agenthub:plan> session latest
agenthub:plan> open latest
agenthub:plan[tx-20260515123000-abcd1234]> watch
agenthub:plan[tx-20260515123000-abcd1234]> explain
agenthub:plan[tx-20260515123000-abcd1234]> effects
agenthub:plan[tx-20260515123000-abcd1234]> memory audit
agenthub:plan[tx-20260515123000-abcd1234]> skills scorecard
agenthub:plan[tx-20260515123000-abcd1234]> undo
```

不离开 shell 检查环境：

```text
agenthub:plan> doctor
agenthub:plan> providers status
agenthub:plan> provider codex
agenthub:plan> providers diagnose codex
agenthub:plan> config show
agenthub:plan> dashboard
```

## 安全性

Shell 使用与 `agenthub run` 相同的 transaction engine：隔离 workspace、command policy、bounded logs、verifier、diff guard、effect ledger、rollback、smart sync、memory promotion 规则和 report。
