# PRD v2 Task 01 — Effect Ledger Foundation

Status: Done

## Goal

Introduce `.agent/tx/<tx-id>/effects.jsonl` as the append-only foundation for effect-aware transactions.

## Acceptance

- Transaction runs write `effects.jsonl`.
- Planned command/process effects are visible before execution policy enforcement completes.
- File effects from the transaction diff are recorded with rollback metadata.
- Verified file effects are marked after verifier success.
- Rolled back file effects are recorded after failure rollback.
- A CLI command can show all effects for a transaction.
- Tests cover successful commit and rollback records.
- README and feature docs are updated in English, Russian, Chinese, and Kazakh.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added the `effects` module and append-only `.agent/tx/<tx-id>/effects.jsonl` records.
- Recorded planned transaction and command/process effects.
- Recorded applied, verified, rollback-pending, and rolled-back file effects from transaction diffs.
- Added `agenthub tx effects <tx-id>` to print the effect ledger.
- Added unit and transaction tests for successful verified effects and rollback effects.
- Updated README and effect-ledger docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: pending.
- Checks: pending.
