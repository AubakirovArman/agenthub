# PRD v2 Task 07 — VCM-OS Typed Memory

Status: Todo

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
