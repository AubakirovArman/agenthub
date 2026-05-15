# Plugin Governance

Languages: [English](plugin-governance.en.md), [Русский](plugin-governance.ru.md), [中文](plugin-governance.zh.md), [Қазақша](plugin-governance.kk.md)

Plugin Governance adds marketplace safety metadata to plugin packages. It works with the existing install/signature flow and writes scorecards under `.agent/plugins/scorecards/`.

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

- `.agent/plugins/installed.json` stores permissions, publisher, review, compatibility, advisories, and scorecard path.
- `.agent/plugins/scorecards/<package>.json` stores manifest validity, signature state, test counts, dangerous permissions, compatibility, trust, and warnings.

Untrusted plugins that request dangerous permissions still require an explicit untrusted override.
