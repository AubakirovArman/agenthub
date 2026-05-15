# PRD v2 Task 12 — Governance v2

Status: Todo

## Goal

Move governance from local policy checks toward central lock precedence, policy bundles, drift detection, and auditable approval workflow artifacts.

## Acceptance

- Add lock precedence model for organization, team, project, and local override policy layers.
- Add central `agent.lock` or `organization.lock` metadata support with drift detection.
- Add policy bundle metadata for secure-code rules, private model routing, sandbox requirements, plugin trust, and raw trace restrictions.
- Add approval workflow artifacts for package install, cloud apply, lock changes, dangerous diff, and raw trace enablement.
- Record approval history in an auditable JSONL artifact.
- Compliance report includes lock/policy differences and approval history summary.
- Existing enterprise policy commands stay compatible.
- Add tests for lock precedence, drift detection, approval history, compliance output, and compatibility.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
