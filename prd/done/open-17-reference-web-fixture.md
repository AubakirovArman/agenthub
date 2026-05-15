# Open Task 17 — Reference Web Fixture

Status: Done

Source: `prd/audit/open/README.md`, `prd.md`

## Goal

End-to-end reference web app fixture for Add Page to Existing Web App.

## Acceptance

- Implementation exists or the PRD gap is explicitly narrowed with shipped behavior.
- README and docs are updated in English, Russian, Chinese, and Kazakh when user-facing behavior changes.
- Tests or smoke checks cover the new behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
- Move this task to `prd/done/` with closing evidence when complete.

## Completed

- Added `examples/reference-web-app`, a self-contained Next.js-style web fixture with dashboard route, reusable dashboard styles, local skill manifests, build script, and dev server.
- Added `examples/reference-web-add-courses.yaml`, an AgentSpec that adds `/courses` through isolated worktree execution, scope limits, build verification, runtime smoke, memory promotion, report, cost, and WAL artifacts.
- Added integration coverage for successful `/courses` end-to-end execution and rollback on an out-of-scope dashboard edit.
- README and feature docs were updated in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `55605bf`.
- Checks: `cargo fmt -- --check`; `scripts/check-module-size.sh 200`; `git diff --check`; `node --check scripts/build.mjs && node --check scripts/server.mjs && npm run build` in `examples/reference-web-app`; `cargo test --test reference_web_fixture`; `cargo test`; `cargo clippy -- -D warnings`; `npm run check` in `editors/vscode`.
