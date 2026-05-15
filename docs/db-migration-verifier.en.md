# DB Migration Verifier

Languages: [English](db-migration-verifier.en.md), [Русский](db-migration-verifier.ru.md), [中文](db-migration-verifier.zh.md), [Қазақша](db-migration-verifier.kk.md)

`db_migration` is a verifier profile for database schema changes. It checks that migration artifacts, schema diff, dry-run output, rollback plan, and seed files are present.

## Use

```yaml
verify:
  profile: db_migration
  commands:
    - npm run db:migrate:dry-run
    - npm run db:seed:check
```

The commands run first. The domain verifier then checks `db/migration.json`.

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

Rules:

- `migrations`, `schema_diff`, `dry_run`, and `seed_files` must point to existing non-empty files.
- `rollback_plan` is required when `rollback_supported` is `true`.
- Paths must be project-relative.

Run the sample:

```bash
agenthub run examples/db-migration-task.yaml
```

Results are written to `.agent/tx/<tx-id>/verifier.json` and `verifier.log`.
