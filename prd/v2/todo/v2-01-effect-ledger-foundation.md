# PRD v2 Task 01 — Effect Ledger Foundation

Status: Todo

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
