# Transaction Watch

`agenthub tx watch` follows the transaction journal for a single transaction:

```bash
agenthub tx watch tx-20260515123000-abcd1234
```

For CI or scripts, use one-shot mode:

```bash
agenthub tx watch tx-20260515123000-abcd1234 --once
```

Output is intentionally compact:

```text
[ok] CREATED transaction created
[ok] RUNNER_READY runner metadata and resource policy recorded
[running] EXECUTING running execution commands
[done] COMMITTED transaction committed
```

`watch` exits automatically when the journal reaches `COMMITTED`, `ROLLED_BACK`, `BLOCKED_ON_HUMAN`, `CANCELED`, or `CLOSED`. Long-running commands also write heartbeat records to `.agent/tx/<tx-id>/heartbeat.jsonl`.
