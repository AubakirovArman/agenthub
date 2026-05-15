# Backend TDD Verifier

Языки: [English](backend-tdd-verifier.en.md), [Русский](backend-tdd-verifier.ru.md), [中文](backend-tdd-verifier.zh.md), [Қазақша](backend-tdd-verifier.kk.md)

`backend_tdd` — verifier profile для backend changes, где нужно доказать наличие tests и API response expectations.

## Использование

```yaml
verify:
  profile: backend_tdd
  commands:
    - cargo test
    - cargo test --test api
```

Сначала выполняются commands. Затем domain verifier проверяет `backend/tdd.json`.

## Manifest

```json
{
  "unit_tests": ["backend/tests/unit/health.test.ts"],
  "integration_tests": ["backend/tests/integration/health.test.ts"],
  "api_responses": [
    {"method": "GET", "path": "/health", "status": 200, "body": {"ok": true}}
  ]
}
```

Правила:

- `unit_tests` и `integration_tests` должны ссылаться на существующие непустые files.
- `api_responses` должен содержать method, route path, HTTP status и inline `body` или `body_path`.
- Paths должны быть project-relative.

Запуск примера:

```bash
agenthub run examples/backend-tdd-task.yaml
```

Результаты пишутся в `.agent/tx/<tx-id>/verifier.json` и `verifier.log`.
