# Effect Ledger

Languages: [English](effect-ledger.en.md), [Русский](effect-ledger.ru.md), [中文](effect-ledger.zh.md), [Қазақша](effect-ledger.kk.md)

AgentHub v2 starts recording transaction effects in `.agent/tx/<tx-id>/effects.jsonl`. The file is append-only JSONL and complements `journal.jsonl`: the journal explains lifecycle state, while the effect ledger lists things the transaction planned, applied, verified, rolled back, or marked as non-rollbackable.

## Records

Each record contains `effect_id`, `effect_type`, `status`, `created_by_node`, rollback metadata, approval metadata, and structured `data`.

Statuses currently written:

- `planned`: transaction plan or command was discovered before execution.
- `applied`: file change appeared in the transaction diff.
- `verified`: file change passed verifier checks.
- `rollback_pending`: failed transaction is about to roll the file effect back.
- `rolled_back`: file effect was rolled back through git worktree cleanup.
- `non_rollbackable`: a process command completed; file changes are tracked separately.

## Usage

```bash
agenthub tx effects tx-...
```

A successful transaction includes verified file effects. A rolled-back transaction includes rollback-pending and rolled-back file effects, and failed memory is written separately instead of promoted.
