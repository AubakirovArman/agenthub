# PRD v2 Task 16 — Hosted / Team Surfaces

Status: Todo

## Goal

Prepare AgentHub for team/self-hosted operation with shared project registry, transaction history, approvals, policies, runners, audit exports, and team dashboard payloads.

## Acceptance

- Add a server/team surface data model for projects, transactions, approvals, policies, runners, costs, memory, and audit summaries.
- Add a local self-hosted export artifact that can be served by a future AgentHub Server.
- Add team dashboard payload generation without requiring a running server.
- Add approval inbox summary across projects where data exists.
- Add runner and policy inventory summaries suitable for admins.
- Add auditor-friendly JSON export for reports/compliance history.
- Keep local-only behavior compatible.
- Add tests for team payload generation, empty/missing project compatibility, approval aggregation, and audit export.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
