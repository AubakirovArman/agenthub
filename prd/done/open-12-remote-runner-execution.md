# Open Task 12 — Remote Runner Execution

Status: Done

Source: `prd/audit/open/README.md`, `prd.md`

## Goal

Real remote runner dispatch and result collection.

## Acceptance

- Implementation exists or the PRD gap is explicitly narrowed with shipped behavior.
- README and docs are updated in English, Russian, Chinese, and Kazakh when user-facing behavior changes.
- Tests or smoke checks cover the new behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
- Move this task to `prd/done/` with closing evidence when complete.

## Completed

- Sandbox Level 2 now selects a configured enterprise remote runner and dispatches execution, repair, reviewer, verifier, and external agent adapter CLI commands through it.
- Sandbox Level 3+ requires a remote runner labeled `enterprise` or `isolated`.
- Remote command results include `remote: true` and the selected runner id.
- `local://` endpoints support local integration tests and single-host deployments; `ssh://host/path` endpoints dispatch through SSH.
- README and feature docs were updated in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `391492b`.
- Checks: `cargo fmt -- --check`; `cargo test`; `cargo clippy -- -D warnings`; `scripts/check-module-size.sh 200`; `git diff --check`; `npm run check` in `editors/vscode`.
