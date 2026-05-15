# Plugin Governance

Языки: [English](plugin-governance.en.md), [Русский](plugin-governance.ru.md), [中文](plugin-governance.zh.md), [Қазақша](plugin-governance.kk.md)

Plugin Governance добавляет marketplace safety metadata к plugin packages. Он работает поверх существующего install/signature flow и пишет scorecards в `.agent/plugins/scorecards/`.

## Поля manifest

```yaml
governance:
  permissions:
    commands: ["cargo test"]
    network: true
    filesystem: ["workspace"]
    models: ["local"]
    workspace_profiles: ["code"]
    verifier_profiles: ["code_build"]
    runtime_packs: ["code.rust"]
  publisher:
    id: publisher.demo
    display: Demo Publisher
  review:
    status: reviewed
    reviewed_by: reviewer.demo
    deprecated: false
  compatibility:
    agenthub_api: "0.1"
  tests:
    - id: golden
      path: tests/golden.txt
  advisories:
    - id: ADV-1
      severity: low
      summary: demo warning
```

## Outputs

- `.agent/plugins/installed.json` хранит permissions, publisher, review, compatibility, advisories и scorecard path.
- `.agent/plugins/scorecards/<package>.json` хранит manifest validity, signature state, test counts, dangerous permissions, compatibility, trust и warnings.

Untrusted plugins с dangerous permissions всё ещё требуют явный untrusted override.
