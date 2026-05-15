# DB Migration Verifier

Языки: [English](db-migration-verifier.en.md), [Русский](db-migration-verifier.ru.md), [中文](db-migration-verifier.zh.md), [Қазақша](db-migration-verifier.kk.md)

`db_migration` — verifier profile для изменений database schema. Он проверяет migration artifacts, schema diff, dry-run output, rollback plan и seed files.

## Использование

```yaml
verify:
  profile: db_migration
  commands:
    - npm run db:migrate:dry-run
    - npm run db:seed:check
```

Сначала выполняются commands. Затем domain verifier проверяет `db/migration.json`.

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

Правила:

- `migrations`, `schema_diff`, `dry_run` и `seed_files` должны указывать на существующие непустые files.
- `rollback_plan` обязателен, когда `rollback_supported` равен `true`.
- Paths должны быть project-relative.

Запуск примера:

```bash
agenthub run examples/db-migration-task.yaml
```

Результаты пишутся в `.agent/tx/<tx-id>/verifier.json` и `verifier.log`.
