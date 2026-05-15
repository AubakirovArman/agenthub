# Plugin Governance

Тілдер: [English](plugin-governance.en.md), [Русский](plugin-governance.ru.md), [中文](plugin-governance.zh.md), [Қазақша](plugin-governance.kk.md)

Plugin Governance plugin packages үшін marketplace safety metadata қосады. Ол existing install/signature flow үстінде жұмыс істейді және scorecards файлдарын `.agent/plugins/scorecards/` ішіне жазады.

## Manifest Fields

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

- `.agent/plugins/installed.json` permissions, publisher, review, compatibility, advisories және scorecard path сақтайды.
- `.agent/plugins/scorecards/<package>.json` manifest validity, signature state, test counts, dangerous permissions, compatibility, trust және warnings сақтайды.

Dangerous permissions сұрайтын untrusted plugins explicit untrusted override талап етеді.
