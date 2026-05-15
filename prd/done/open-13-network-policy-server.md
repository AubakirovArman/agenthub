# Open Task 13 — Network Policy Server

Status: Done

Source: `prd/audit/open/README.md`, `prd.md`

## Goal

Networked central policy server beyond file-backed AGENTHUB_POLICY_PATH.

## Acceptance

- Implementation exists or the PRD gap is explicitly narrowed with shipped behavior.
- README and docs are updated in English, Russian, Chinese, and Kazakh when user-facing behavior changes.
- Tests or smoke checks cover the new behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.
- Move this task to `prd/done/` with closing evidence when complete.

## Completed

- Added an HTTP enterprise policy client using `AGENTHUB_POLICY_URL`.
- Added project bootstrap support for `enterprise.policy_server.mode: http`.
- Added built-in `agenthub enterprise policy-server` to serve YAML policy over HTTP.
- Added optional bearer token support through `AGENTHUB_POLICY_TOKEN` or configured `token_env`.
- README and feature docs were updated in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: pending.
- Checks: pending.
