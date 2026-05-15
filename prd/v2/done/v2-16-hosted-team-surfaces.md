# PRD v2 Task 16 — Hosted / Team Surfaces

Status: Done

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

## Completed

- Added `agenthub::team` data model for project, transaction, approval, policy, runner, memory, analytics, audit, and report summaries.
- Added self-hosted export artifacts: `team_payload.json` and `audit_export.json`.
- Added dashboard integration so `agenthub dashboard` also writes `.agent/reports/team/` exports for the current project.
- Added approval inbox aggregation across project paths.
- Added policy and runner inventory summaries for admin/team payloads.
- Added auditor-friendly JSON export for audit events and transaction/compliance report links.
- Preserved local-only behavior and missing-project compatibility.
- Added tests for payload generation, missing project compatibility, approval aggregation, and audit export.
- Added Hosted/Team Surfaces docs in English, Russian, Chinese, and Kazakh and updated README links.

## Evidence

- Implementation commit: `28257fa Complete hosted team surfaces task`
- `cargo fmt -- --check`
- `scripts/check-module-size.sh 200`
- `git diff --check`
- `cargo test team::tests`
- `cargo clippy -- -D warnings`
- `cargo test`
- `npm run check` in `editors/vscode`
