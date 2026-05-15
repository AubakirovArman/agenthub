# PRD v2 Task 04 — Smart Sync

Status: Todo

## Goal

Replace the current coarse HEAD-only sync block with file-level overlap detection and a safe rebase path for independent changes.

## Acceptance

- Transaction baseline captures relevant file hashes.
- Sync check distinguishes independent main changes from overlapping transaction changes.
- Overlap blocks on human with structured report data.
- Independent changes can rebase transaction branch onto current main.
- Diff guard and verifier run again after rebase before commit.
- Report shows the sync decision and rerun verifier result.
- Tests cover independent rebase and overlapping block behavior.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
