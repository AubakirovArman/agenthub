# Effect Ledger

Тілдер: [English](effect-ledger.en.md), [Русский](effect-ledger.ru.md), [中文](effect-ledger.zh.md), [Қазақша](effect-ledger.kk.md)

AgentHub v2 transaction effects жазбасын `.agent/tx/<tx-id>/effects.jsonl` ішіне жаза бастайды. Бұл `journal.jsonl` жанындағы append-only JSONL: journal lifecycle state түсіндіреді, ал effect ledger transaction planned, applied, verified, rolled back немесе non-rollbackable деп белгіленген нәрселерді тізеді.

## Жазбалар

Әр record ішінде `effect_id`, `effect_type`, `status`, `created_by_node`, rollback metadata, approval metadata және structured `data` бар.

Қазіргі statuses:

- `planned`: transaction plan немесе command execution алдында анықталды.
- `applied`: file change transaction diff ішінде пайда болды.
- `verified`: file change verifier checks арқылы өтті.
- `rollback_pending`: failed transaction file effect rollback жасауға дайын.
- `rolled_back`: file effect git worktree cleanup арқылы rollback болды.
- `non_rollbackable`: process command аяқталды; file changes бөлек бақыланады.

## Қолдану

```bash
agenthub tx effects tx-...
```

Сәтті transaction ішінде verified file effects болады. Rollback болған transaction rollback-pending және rolled-back file effects сақтайды, ал failed memory бөлек жазылып, promoted болмайды.
