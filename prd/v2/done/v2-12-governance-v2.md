# PRD v2 Task 12 — Governance v2

Status: Done

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

## Completed

- Added governance lock evaluation for organization, team, project, and local override layers.
- Added central lock metadata, policy bundle merge precedence, local override drift detection, and local override exclusion when central locks disallow it.
- Added auditable approval JSONL records and summary aggregation.
- Added compliance report governance details with effective bundles, drift findings, and approval history by kind/status.
- Preserved existing enterprise policy behavior and command compatibility.
- Added Governance v2 docs in English, Russian, Chinese, and Kazakh and linked them from all README variants.

## Evidence

- Implementation commit: `c1e9836 Complete governance v2 task`
- `cargo fmt -- --check`
- `scripts/check-module-size.sh 200`
- `git diff --check`
- `cargo test enterprise::tests`
- `cargo clippy -- -D warnings`
- `cargo test`
- `npm run check` in `editors/vscode`
