# Open Task 08 — Backend TDD Verifier

Status: Done

Closing evidence: implementation commit `52668be`; verified with `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo test backend_tdd`, `scripts/check-module-size.sh 200`, `git diff --check`, and `npm run check` in `editors/vscode/`.

Source: `prd/audit/open/README.md`, `prd.md`

## Goal

Specialized backend_tdd verifier profile.

## Acceptance

- [x] Implementation exists: `backend_tdd` verifier profile with TDD manifest, unit/integration test file checks, and API response expectation checks.
- [x] README and docs are updated in English, Russian, Chinese, and Kazakh.
- [x] Tests and smoke checks cover domain verifier behavior and transaction execution.
- [x] Module-size check stays under 200 lines per Rust/JS implementation file.
- [x] Task moved to `prd/done/` with closing evidence.
