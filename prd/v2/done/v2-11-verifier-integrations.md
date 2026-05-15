# PRD v2 Task 11 — Verifier Integrations v2

Status: Done

## Goal

Move verifier output from command/domain checks toward a structured verifier integration layer with reusable check records, trendable artifacts, and memory feedback.

## Acceptance

- Add a structured verifier check schema that can represent code, infra, data, content, media, and research checks.
- Convert existing command, runtime smoke, and domain verifier outputs into a unified structured JSON artifact.
- Add verifier fingerprints for failed checks and feed them into failed-attempt memory or typed warning memory.
- Add verifier trend artifact data suitable for dashboard/analytics consumption.
- Keep existing verifier command behavior and domain profiles compatible.
- Make verifier integrations plugin-compatible where local plugin verifier metadata already exists.
- Report includes a structured verifier summary and trend/fingerprint references.
- Add tests for structured verifier JSON, failure fingerprints, plugin-compatible verifier metadata, report output, and compatibility.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added `verifier_integration.json` as a unified structured verifier artifact while preserving legacy `verifier.json`.
- Added structured check records for command verifiers, runtime smoke checks, and domain verifier profiles.
- Added verifier fingerprints for failed checks and included the fingerprint in rollback failed-attempt memory reasons.
- Added verifier trend fields with total, passed, failed, and per-category counts.
- Added plugin compatibility metadata by matching installed verifier plugin profiles from `.agent/plugins/installed.json`.
- Added structured verifier summary, fingerprint list, and artifact references to transaction reports.
- Split verifier report rendering into `src/report/markdown/verifier_section.rs` to keep module sizes under 200 lines.
- Added tests for structured verifier artifacts, fingerprint generation, plugin verifier metadata compatibility, report output, and failed-attempt memory feedback.
- Updated README and verifier integration docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `21acaeb`.
- Checks: `cargo fmt -- --check`; `scripts/check-module-size.sh 200`; `git diff --check`; `cargo test verifier::integration_tests`; `cargo test repair_attempts_are_bounded_when_unresolved`; `cargo clippy -- -D warnings`; `cargo test`; `npm run check` in `editors/vscode`.
