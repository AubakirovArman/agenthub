# PRD v2 Task 13 — Analytics History

Status: Todo

## Goal

Move from per-transaction reports to persisted trend intelligence that can be viewed locally and exported to external analytics tools.

## Acceptance

- Persist metrics history across transactions under `.agent/metrics/`.
- Record success rate, rollback rate, repair rate, human-block frequency, average time to commit, and dangerous diff rate.
- Record model, topology, verifier, skill, and task-type metrics when those artifacts exist.
- Add JSONL and CSV exports for analytics history.
- Make dashboard/report output trend-ready instead of only showing the latest transaction.
- Keep metrics append-only or explicitly snapshot-versioned so history survives process restarts.
- Add tests for metric recording, trend aggregation, export files, and missing-artifact compatibility.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
