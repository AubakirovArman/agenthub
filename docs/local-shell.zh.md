# 本地 Shell

AgentHub 可以作为交互式本地 shell 运行：

```bash
agenthub
# 或
agenthub shell
```

这个 shell 面向 local-first 使用。你可以查看历史事务会话、打开报告、用自然语言生成 draft AgentSpec，并在同一个 prompt 中运行 spec。

## 命令

```text
help                         显示命令
init                         初始化 .agent
sessions                     列出最近事务
open <tx-id>                 打开事务报告并设为当前事务
watch [tx-id]                跟随实时 transaction journal
cancel [tx-id]               请求取消 transaction
report [tx-id]               打印报告，默认使用当前事务
effects [tx-id]              打印 effect ledger
ask <request>                写入 draft AgentSpec
do <request>                 写入 draft 并立即执行
run <spec|request> [--no-commit]
quit                         退出
普通文本                     等同于 ask <request>
```

## 示例

从消息创建 draft：

```text
agenthub> add /courses page in the dashboard style
draft .agent/drafts/shell-20260515123000.yaml
```

运行 spec：

```text
agenthub> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

立即运行自然语言请求：

```text
agenthub> do add a generated health-check file
```

浏览历史会话：

```text
agenthub> sessions
agenthub> open tx-20260515123000-abcd1234
agenthub[tx-20260515123000-abcd1234]> watch
agenthub[tx-20260515123000-abcd1234]> effects
```

## 安全性

Shell 使用与 `agenthub run` 相同的 transaction engine：隔离 workspace、command policy、bounded logs、verifier、diff guard、effect ledger、rollback、smart sync、memory promotion 规则和 report。
