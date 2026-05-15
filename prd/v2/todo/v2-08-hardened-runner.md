# PRD v2 Task 08 — Hardened Runner

Status: Todo

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
