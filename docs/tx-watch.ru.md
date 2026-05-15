# Наблюдение за транзакцией

`agenthub tx watch` следит за journal одной транзакции:

```bash
agenthub tx watch tx-20260515123000-abcd1234
```

Для CI и скриптов есть одноразовый режим:

```bash
agenthub tx watch tx-20260515123000-abcd1234 --once
```

Вывод специально короткий:

```text
[ok] CREATED transaction created
[ok] RUNNER_READY runner metadata and resource policy recorded
[running] EXECUTING running execution commands
[done] COMMITTED transaction committed
```

`watch` сам завершается, когда journal доходит до `COMMITTED`, `ROLLED_BACK`, `BLOCKED_ON_HUMAN` или `CLOSED`.
