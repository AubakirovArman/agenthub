# Transaction Explain

`agenthub tx explain <tx-id>` 会把 transaction artifacts 转成简短、可读的解释。

它会读取 `.agent/tx/<tx-id>/journal.jsonl`、`diff_guard.json`、`verifier.json`、`sync.json`、`effects.jsonl`、`command_policy.json` 和 `report.md`，如果这些文件存在。

## 用法

```bash
agenthub tx explain tx-20260515123000-abcd1234
```

在本地 shell 中：

```text
agenthub:plan> open latest
agenthub:plan[tx-...]> explain
```

## 输出

输出有四个部分：

```text
Why
What Happened
Next
Artifacts
```

Diff guard failure 会说明违反的 scope 规则，并建议修改任务或 `scope.allow` / `scope.deny`。Verifier failure 会指向 `verifier.log` 和 command log files。Smart sync overlap 会列出冲突文件，并提示先手动解决再 resume。
