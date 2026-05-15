# Effect Ledger

Языки: [English](effect-ledger.en.md), [Русский](effect-ledger.ru.md), [中文](effect-ledger.zh.md), [Қазақша](effect-ledger.kk.md)

AgentHub v2 начинает записывать transaction effects в `.agent/tx/<tx-id>/effects.jsonl`. Это append-only JSONL рядом с `journal.jsonl`: journal объясняет lifecycle state, а effect ledger перечисляет, что транзакция planned, applied, verified, rolled back или пометила как non-rollbackable.

## Записи

Каждая запись содержит `effect_id`, `effect_type`, `status`, `created_by_node`, rollback metadata, approval metadata и structured `data`.

Текущие statuses:

- `planned`: transaction plan или command найден до исполнения.
- `applied`: file change появился в transaction diff.
- `verified`: file change прошёл verifier checks.
- `rollback_pending`: failed transaction готовится откатить file effect.
- `rolled_back`: file effect откатан через git worktree cleanup.
- `non_rollbackable`: process command завершился; file changes отслеживаются отдельно.

## Использование

```bash
agenthub tx effects tx-...
```

Успешная транзакция содержит verified file effects. Откатившаяся транзакция содержит rollback-pending и rolled-back file effects, а failed memory пишется отдельно и не promoted.
