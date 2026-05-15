# PRD v2 Task 02 — Rollback Handlers

Status: Todo

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
