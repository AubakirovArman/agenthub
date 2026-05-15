# Backend TDD Verifier

Тілдер: [English](backend-tdd-verifier.en.md), [Русский](backend-tdd-verifier.ru.md), [中文](backend-tdd-verifier.zh.md), [Қазақша](backend-tdd-verifier.kk.md)

`backend_tdd` — backend changes үшін verifier profile. Ол tests және API response expectations бар екенін дәлелдейді.

## Қолдану

```yaml
verify:
  profile: backend_tdd
  commands:
    - cargo test
    - cargo test --test api
```

Алдымен commands орындалады. Содан кейін domain verifier `backend/tdd.json` тексереді.

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

Ережелер:

- `unit_tests` және `integration_tests` бар әрі бос емес files көрсетуі керек.
- `api_responses` method, route path, HTTP status және inline `body` немесе `body_path` қамтуы керек.
- Paths project-relative болуы керек.

Мысалды іске қосу:

```bash
agenthub run examples/backend-tdd-task.yaml
```

Нәтижелер `.agent/tx/<tx-id>/verifier.json` және `verifier.log` ішіне жазылады.
