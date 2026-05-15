# PRD v2 Task 08 — Hardened Runner

Status: Done

## Goal

Improve local and remote runner safety with resource-limit metadata, cancellation hooks, structured artifacts, and platform-specific process controls where available.

## Acceptance

- Add runner capability and trust-level metadata for local and remote execution.
- Record resource-limit policy for CPU, memory, time, disk, network, and filesystem access.
- Add structured command resource usage fields where available.
- Add cancellation/cancel-request artifact support for transactions.
- Improve process-tree cleanup documentation and platform-specific runner reporting.
- Keep current local and ssh runner behavior compatible.
- Tests cover runner metadata, timeout/cancel artifacts, and transaction report integration.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added `RunnerMetadata`, `ResourceLimitPolicy`, and `ResourceUsage` to local and remote command results.
- Added trust-level, platform, process-control, capability, resource-limit, and runner endpoint metadata.
- Added transaction-level `runner.json` and `cancel_status.json` artifacts.
- Added `cancel_request.json` read/write support and command-loop cancellation checks before each command starts.
- Split platform process-control into a dedicated module and documented Unix process-group cleanup plus fallback behavior.
- Kept local, `local://`, and `ssh://` runner behavior compatible.
- Added tests for runner metadata, timeout/process cleanup, cancel artifacts, and transaction report integration.
- Updated README and hardened-runner docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: pending.
- Checks: pending.
