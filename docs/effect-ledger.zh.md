# Effect Ledger

语言: [English](effect-ledger.en.md), [Русский](effect-ledger.ru.md), [中文](effect-ledger.zh.md), [Қазақша](effect-ledger.kk.md)

AgentHub v2 开始把 transaction effects 写入 `.agent/tx/<tx-id>/effects.jsonl`。这是 append-only JSONL，和 `journal.jsonl` 互补：journal 说明 lifecycle state，effect ledger 列出 transaction planned、applied、verified、rolled back 或标记为 non-rollbackable 的内容。

## 记录

每条记录包含 `effect_id`、`effect_type`、`status`、`created_by_node`、rollback metadata、approval metadata 和 structured `data`。

当前写入的 statuses：

- `planned`: transaction plan 或 command 在执行前被发现。
- `applied`: file change 出现在 transaction diff 中。
- `verified`: file change 通过 verifier checks。
- `rollback_pending`: failed transaction 即将 rollback 该 file effect。
- `rolled_back`: file effect 已通过 git worktree cleanup rollback。
- `non_rollbackable`: process command 已完成；file changes 会单独跟踪。

## 使用

```bash
agenthub tx effects tx-...
```

成功事务包含 verified file effects。回滚事务包含 rollback-pending 和 rolled-back file effects，failed memory 会单独写入，不会 promoted。
