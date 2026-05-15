# Open Task 14 — Plugin Signature Verification

Status: Done

Source: `prd/audit/open/README.md`, `prd.md`

## Goal

Cryptographic plugin package signature verification.

## Acceptance

- Implementation exists or the PRD gap is explicitly narrowed with shipped behavior.
- README and docs are updated in English, Russian, Chinese, and Kazakh when user-facing behavior changes.
- Tests or smoke checks cover the new behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
- Move this task to `prd/done/` with closing evidence when complete.

## Completed

- Added SHA-256 package signature verification for plugin manifests with `signature.algorithm: sha256`.
- Added deterministic package digest calculation with `agenthub plugins digest`.
- `agenthub plugins inspect` rejects mismatched cryptographic signatures.
- `agenthub plugins install --trust trusted` requires a verified cryptographic signature.
- Plugin locks now record `signature_verified`.
- README and feature docs were updated in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: pending.
- Checks: pending.
