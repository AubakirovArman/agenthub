# Backend TDD Verifier

语言: [English](backend-tdd-verifier.en.md), [Русский](backend-tdd-verifier.ru.md), [中文](backend-tdd-verifier.zh.md), [Қазақша](backend-tdd-verifier.kk.md)

`backend_tdd` 是用于 backend changes 的 verifier profile，用来证明已经产出 tests 和 API response expectations。

## 使用

```yaml
verify:
  profile: backend_tdd
  commands:
    - cargo test
    - cargo test --test api
```

commands 会先运行。随后 domain verifier 检查 `backend/tdd.json`。

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

规则：

- `unit_tests` 和 `integration_tests` 必须引用存在且非空的 files。
- `api_responses` 必须包含 method、route path、HTTP status，以及 inline `body` 或 `body_path`。
- Paths 必须是 project-relative。

运行示例：

```bash
agenthub run examples/backend-tdd-task.yaml
```

结果写入 `.agent/tx/<tx-id>/verifier.json` 和 `verifier.log`。
