# Hosted / Team Surfaces

Languages: [English](hosted-team-surfaces.en.md), [Русский](hosted-team-surfaces.ru.md), [中文](hosted-team-surfaces.zh.md), [Қазақша](hosted-team-surfaces.kk.md)

Hosted / Team Surfaces provide local self-hosted export artifacts for a future AgentHub Server. They do not require a running server.

## Artifacts

`agenthub dashboard` also writes:

```text
.agent/reports/team/team_payload.json
.agent/reports/team/audit_export.json
```

`team_payload.json` contains project summaries for transactions, approvals, policy source, runners, memory, analytics cost, audit counts, and report links.

`audit_export.json` contains auditor-friendly audit events and transaction/compliance report links.

The Rust API `agenthub::team::write_export()` can generate the same artifacts for multiple project paths.
