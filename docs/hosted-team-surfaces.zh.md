# Hosted / Team Surfaces

语言: [English](hosted-team-surfaces.en.md), [Русский](hosted-team-surfaces.ru.md), [中文](hosted-team-surfaces.zh.md), [Қазақша](hosted-team-surfaces.kk.md)

Hosted / Team Surfaces 为未来的 AgentHub Server 提供本地 self-hosted export artifacts。不需要运行 server。

## Artifacts

`agenthub dashboard` 也会写入:

```text
.agent/reports/team/team_payload.json
.agent/reports/team/audit_export.json
```

`team_payload.json` 包含 project summaries: transactions、approvals、policy source、runners、memory、analytics cost、audit counts 和 report links。

`audit_export.json` 包含 auditor-friendly audit events，以及 transaction/compliance report links。

Rust API `agenthub::team::write_export()` 可以为多个 project paths 生成同样的 artifacts。
