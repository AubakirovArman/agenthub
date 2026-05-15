# Backend TDD Verifier

Languages: [English](backend-tdd-verifier.en.md), [Русский](backend-tdd-verifier.ru.md), [中文](backend-tdd-verifier.zh.md), [Қазақша](backend-tdd-verifier.kk.md)

`backend_tdd` is a verifier profile for backend changes that must prove tests and API response expectations were produced.

## Use

```yaml
verify:
  profile: backend_tdd
  commands:
    - cargo test
    - cargo test --test api
```

The commands run first. The domain verifier then checks `backend/tdd.json`.

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

Rules:

- `unit_tests` and `integration_tests` must reference existing non-empty files.
- `api_responses` must include method, route path, HTTP status, and either inline `body` or a `body_path`.
- Paths must be project-relative.

Run the sample:

```bash
agenthub run examples/backend-tdd-task.yaml
```

Results are written to `.agent/tx/<tx-id>/verifier.json` and `verifier.log`.
