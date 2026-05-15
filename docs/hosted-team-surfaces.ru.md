# Hosted / Team Surfaces

Языки: [English](hosted-team-surfaces.en.md), [Русский](hosted-team-surfaces.ru.md), [中文](hosted-team-surfaces.zh.md), [Қазақша](hosted-team-surfaces.kk.md)

Hosted / Team Surfaces дают локальные self-hosted export artifacts для будущего AgentHub Server. Запущенный server не нужен.

## Артефакты

`agenthub dashboard` также пишет:

```text
.agent/reports/team/team_payload.json
.agent/reports/team/audit_export.json
```

`team_payload.json` содержит project summaries: transactions, approvals, policy source, runners, memory, analytics cost, audit counts и report links.

`audit_export.json` содержит auditor-friendly audit events и ссылки на transaction/compliance reports.

Rust API `agenthub::team::write_export()` может создать такие же artifacts для нескольких project paths.
