# PRD v2 Task 03 — Resume, Retry, Resolve

Status: Done

## Goal

Make blocked and failed transactions actionable instead of read-only historical reports.

## Acceptance

- `agenthub tx resolve <tx-id> --note ...` records a human resolution note.
- `agenthub tx resume <tx-id>` can continue supported blocked states.
- `agenthub tx retry <tx-id> --from <state>` creates a controlled retry plan.
- Resume/retry/resolve events are written to journal, WAL, and effect ledger.
- Failed external effects do not promote committed memory during resume/retry flows.
- Tests cover resolve metadata and at least one supported resume or retry path.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added `agenthub tx resolve <tx-id> --note ...` and `resolutions.jsonl`.
- Added `agenthub tx retry <tx-id> --from <state>` and controlled retry plan artifacts.
- Added `agenthub tx resume <tx-id>` for resolved `BLOCKED_ON_HUMAN` transactions.
- Resume creates `resume-plan.yaml`, sets `approval_required=true`, runs a linked new transaction, and writes `resume.json`.
- Resolve, retry, and resume write journal events, WAL records, and effect ledger control events.
- Added unit tests for resolve/retry artifacts and an integration test for resolving and resuming a blocked transaction.
- Updated README and resume/retry docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `624264d`.
- Checks: `cargo fmt -- --check`; `scripts/check-module-size.sh 200`; `git diff --check`; `cargo test tx_control`; `cargo test resolve_and_resume_blocked_transaction`; `cargo clippy -- -D warnings`; `cargo test`; `npm run check` in `editors/vscode`.
