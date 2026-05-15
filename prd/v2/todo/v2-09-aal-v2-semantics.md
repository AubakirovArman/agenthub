# PRD v2 Task 09 — AAL v2 Semantics

Status: Todo

## Goal

Move AAL from parser-only DSL toward a versioned language with semantic diagnostics, imports, compatibility checks, and live-validation friendly output.

## Acceptance

- Add `aal "0.2"` version handling while preserving existing AAL examples.
- Add import declarations for skills/rules with semantic validation stubs.
- Add semantic diagnostics for unknown skills, unknown verifier profiles, workspace/skill incompatibility, policy conflicts, and route smoke preconditions.
- Add structured diagnostics output suitable for editor/LSP use.
- Add formatter or normalized rendering for parsed AAL.
- Add tests for parser compatibility, semantic errors, and valid v0.2 examples.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
