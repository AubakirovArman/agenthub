# Plugin Governance

语言: [English](plugin-governance.en.md), [Русский](plugin-governance.ru.md), [中文](plugin-governance.zh.md), [Қазақша](plugin-governance.kk.md)

Plugin Governance 为 plugin packages 增加 marketplace safety metadata。它复用现有 install/signature flow，并把 scorecards 写入 `.agent/plugins/scorecards/`。

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

- `.agent/plugins/installed.json` stores permissions、publisher、review、compatibility、advisories 和 scorecard path。
- `.agent/plugins/scorecards/<package>.json` stores manifest validity、signature state、test counts、dangerous permissions、compatibility、trust 和 warnings。

请求 dangerous permissions 的 untrusted plugins 仍然需要 explicit untrusted override。
