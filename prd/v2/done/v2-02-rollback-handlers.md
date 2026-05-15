# PRD v2 Task 02 — Rollback Handlers

Status: Done

## Goal

Add concrete rollback handler records and handler execution beyond the current git worktree cleanup foundation.

## Acceptance

- File snapshot or manifest restore handlers are represented as concrete handler types.
- Dependency manifest and lockfile changes can be restored or marked post-commit pending.
- Non-rollbackable effects require an explicit reason.
- Rollback results are written to effect ledger and transaction report artifacts.
- Tests cover rollback handler selection and failure behavior.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added rollback handler selection for normal files, package manifests/lockfiles, Terraform state files, and environment/secret-like files.
- Added `.agent/tx/<tx-id>/rollback.json` for rolled-back transactions.
- Updated effect ledger file effects to record concrete rollback handler names.
- Added unit tests for handler selection/report writing and transaction coverage for rollback artifacts.
- Updated README and rollback handler docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `f57647f`.
- Checks: `cargo fmt -- --check`; `scripts/check-module-size.sh 200`; `git diff --check`; `cargo test rollback`; `cargo test failed_transaction_rolls_back_and_records_failed_attempt`; `cargo clippy -- -D warnings`; `cargo test`; `npm run check` in `editors/vscode`.
