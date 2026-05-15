# DB Migration Verifier

语言: [English](db-migration-verifier.en.md), [Русский](db-migration-verifier.ru.md), [中文](db-migration-verifier.zh.md), [Қазақша](db-migration-verifier.kk.md)

`db_migration` 是用于 database schema changes 的 verifier profile。它检查 migration artifacts、schema diff、dry-run output、rollback plan 和 seed files。

## 使用

```yaml
verify:
  profile: db_migration
  commands:
    - npm run db:migrate:dry-run
    - npm run db:seed:check
```

commands 会先运行。随后 domain verifier 检查 `db/migration.json`。

## Manifest

```json
{
  "migrations": ["db/migrations/001_create_users.sql"],
  "schema_diff": "db/schema.diff",
  "dry_run": "db/dry-run.log",
  "rollback_supported": true,
  "rollback_plan": "db/rollback.sql",
  "seed_files": ["db/seeds/users.sql"]
}
```

规则：

- `migrations`、`schema_diff`、`dry_run` 和 `seed_files` 必须指向存在且非空的 files。
- 当 `rollback_supported` 为 `true` 时，必须提供 `rollback_plan`。
- Paths 必须是 project-relative。

运行示例：

```bash
agenthub run examples/db-migration-task.yaml
```

结果写入 `.agent/tx/<tx-id>/verifier.json` 和 `verifier.log`。
