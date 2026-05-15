# PRD v2 Task 07 — VCM-OS Typed Memory

Status: Done

## Goal

Move VCM memory from mostly raw JSONL records toward typed project intelligence with schemas, views, retrieval filters, and integrity checks.

## Acceptance

- Add `.agent/schemas/` memory schemas for core, code, infra, data, and content domains.
- Add typed memory records for architecture decisions, dependency policy, routes, known failures, and domain-specific facts.
- Add schema-filtered retrieval for context pack construction while preserving recent-memory fallback.
- Add compaction views under `.agent/memory/views/`.
- Add supersession/staleness metadata and a memory audit artifact.
- Failed attempts remain warnings and are not promoted as current truth.
- Tests cover typed memory write, retrieval filtering, compaction views, and failed-attempt separation.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added core/code/content/data/infra `.memory.yaml` schemas under `.agent/schemas/`.
- Added typed memory metadata fields: schema, status, supersedes, stale, confidence, and last verified commit.
- Added `write_typed_fact` and schema-filtered `retrieve_relevant` with recent-memory fallback.
- Switched context pack memory retrieval to domain-aware typed retrieval.
- Added `.agent/memory/views/` current-truth views and `.agent/memory/audit.json`.
- Kept failed attempts in `failed_attempts.jsonl` and `known_failures.json` as warning-only memory.
- Added tests for typed memory write/retrieval/views and failed-attempt separation.
- Updated README and VCM-OS memory docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: pending.
- Checks: pending.
