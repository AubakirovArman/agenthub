# Transaction Watch

`agenthub tx watch` бір transaction journal-ын live бақылайды:

```bash
agenthub tx watch tx-20260515123000-abcd1234
```

CI немесе script үшін one-shot режимі бар:

```bash
agenthub tx watch tx-20260515123000-abcd1234 --once
```

Шығыс қысқа форматта беріледі:

```text
[ok] CREATED transaction created
[ok] RUNNER_READY runner metadata and resource policy recorded
[running] EXECUTING running execution commands
[done] COMMITTED transaction committed
```

Journal `COMMITTED`, `ROLLED_BACK`, `BLOCKED_ON_HUMAN`, `CANCELED` немесе `CLOSED` күйіне жеткенде `watch` өзі аяқталады. Ұзақ commands heartbeat records-ты `.agent/tx/<tx-id>/heartbeat.jsonl` ішіне де жазады.
