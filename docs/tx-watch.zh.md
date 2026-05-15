# Transaction Watch

`agenthub tx watch` 用来跟随单个事务的 journal：

```bash
agenthub tx watch tx-20260515123000-abcd1234
```

CI 或脚本可以使用 one-shot 模式：

```bash
agenthub tx watch tx-20260515123000-abcd1234 --once
```

输出保持紧凑：

```text
[ok] CREATED transaction created
[ok] RUNNER_READY runner metadata and resource policy recorded
[running] EXECUTING running execution commands
[done] COMMITTED transaction committed
```

当 journal 到达 `COMMITTED`、`ROLLED_BACK`、`BLOCKED_ON_HUMAN` 或 `CLOSED` 时，`watch` 会自动退出。
