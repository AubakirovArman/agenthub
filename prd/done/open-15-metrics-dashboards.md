# Open Task 15 — Metrics Dashboards

Status: Done

Source: `prd/audit/open/README.md`, `prd.md`

## Goal

Aggregated reliability, context, quality, trust, and cost metrics dashboards.

## Acceptance

- Implementation exists or the PRD gap is explicitly narrowed with shipped behavior.
- README and docs are updated in English, Russian, Chinese, and Kazakh when user-facing behavior changes.
- Tests or smoke checks cover the new behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
- Move this task to `prd/done/` with closing evidence when complete.

## Completed

- Added aggregated `metrics` payload to Web Dashboard `data.json` and `data.js`.
- Added Metrics Dashboard UI panel for reliability, context, quality, trust, and cost KPI groups.
- Aggregates transaction status rates, verifier/reviewer pass rates, memory/context data, plugin trust/signature data, and cost/token totals.
- README and feature docs were updated in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `17d6e2b`.
- Checks: `cargo fmt -- --check`; `cargo test`; `cargo clippy -- -D warnings`; `scripts/check-module-size.sh 200`; `git diff --check`; `npm run check` in `editors/vscode`; targeted `cargo test web_dashboard`; `node --check src/web_dashboard/assets/dashboard.js`.
