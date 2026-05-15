# Resume, Retry, Resolve

Языки: [English](resume-retry.en.md), [Русский](resume-retry.ru.md), [中文](resume-retry.zh.md), [Қазақша](resume-retry.kk.md)

AgentHub v2 делает blocked transactions actionable. Транзакция может получить human resolution note, создать controlled retry plan или resume поддержанного состояния `BLOCKED_ON_HUMAN`.

## Команды

```bash
agenthub tx resolve tx-... --note "Approved package install"
agenthub tx retry tx-... --from VERIFYING
agenthub tx resume tx-...
```

`resolve` дописывает `.agent/tx/<tx-id>/resolutions.jsonl` и пишет `RESOLVED` в journal и WAL. `retry` копирует исходный `plan.yaml` в controlled retry plan и пишет `retry_plan.json`. `resume` сейчас поддерживает blocked transactions с resolution note: создаёт `resume-plan.yaml`, ставит `approval_required=true` и запускает связанную новую транзакцию.

Все три команды также пишут control effects в `effects.jsonl`.
