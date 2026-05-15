# DB Migration Verifier

Тілдер: [English](db-migration-verifier.en.md), [Русский](db-migration-verifier.ru.md), [中文](db-migration-verifier.zh.md), [Қазақша](db-migration-verifier.kk.md)

`db_migration` — database schema changes үшін verifier profile. Ол migration artifacts, schema diff, dry-run output, rollback plan және seed files тексереді.

## Қолдану

```yaml
verify:
  profile: db_migration
  commands:
    - npm run db:migrate:dry-run
    - npm run db:seed:check
```

Алдымен commands орындалады. Содан кейін domain verifier `db/migration.json` тексереді.

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

Ережелер:

- `migrations`, `schema_diff`, `dry_run` және `seed_files` бар әрі бос емес files көрсетуі керек.
- `rollback_supported` мәні `true` болса, `rollback_plan` міндетті.
- Paths project-relative болуы керек.

Мысалды іске қосу:

```bash
agenthub run examples/db-migration-task.yaml
```

Нәтижелер `.agent/tx/<tx-id>/verifier.json` және `verifier.log` ішіне жазылады.
