# Hosted / Team Surfaces

Тілдер: [English](hosted-team-surfaces.en.md), [Русский](hosted-team-surfaces.ru.md), [中文](hosted-team-surfaces.zh.md), [Қазақша](hosted-team-surfaces.kk.md)

Hosted / Team Surfaces болашақ AgentHub Server үшін жергілікті self-hosted export artifacts береді. Running server қажет емес.

## Artifacts

`agenthub dashboard` қосымша мыналарды жазады:

```text
.agent/reports/team/team_payload.json
.agent/reports/team/audit_export.json
```

`team_payload.json` ішінде project summaries бар: transactions, approvals, policy source, runners, memory, analytics cost, audit counts және report links.

`audit_export.json` auditor-friendly audit events және transaction/compliance report links сақтайды.

Rust API `agenthub::team::write_export()` бірнеше project paths үшін дәл осындай artifacts жасай алады.
