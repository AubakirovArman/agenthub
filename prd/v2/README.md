# AgentHub PRD v2

AgentHub v1 is the transactional runtime foundation. AgentHub v2 hardens that foundation into a memory-aware, provider-controlled, policy-governed agent platform.

## North Star

AgentHub v2 must safely execute agent tasks, manage external effects, route models through a real gateway, deepen VCM-OS memory, measure quality, and prepare for team/hosted operation.

## Priority Order

1. Priority 0: brand audit and naming consistency.
2. Execution Kernel v2: effect-aware transactions, rollback, resume/retry, smart sync, post-commit effects.
3. Full LLM Provider Gateway: provider abstraction, streaming, retry/backoff, rate limits, budgets, failover.
4. VCM-OS v2: typed schemas, retrieval, compaction views, stale/superseded detection.
5. Hardened Runner: resource limits, runner daemon, cancellation, artifacts, platform-specific process control.
6. AAL v2 and LSP: formal spec, versioning, semantic diagnostics, formatter, VS Code validation.
7. Hosted/team surfaces after execution, memory, gateway, and sandbox hardening.

## Track Index

| Track | Goal |
|---|---|
| 1. Execution Kernel v2 | Effect-aware transactions with rollback and resumability. |
| 2. Smart Sync | Rebase independent main changes and block only overlapping changes. |
| 3. Workspace Runtime v2 | Real pluggable workspaces behind a runtime trait. |
| 4. VCM-OS v2 | Typed project intelligence instead of raw JSONL memory only. |
| 5. AAL v2 | Formal language, type checks, imports, versioning, and LSP. |
| 6. Full LLM Gateway | Unified model control plane for API and CLI providers. |
| 7. Adaptive Orchestration | Dynamic topology/model choice from task type and history. |
| 8. Verifier Integrations v2 | Structured verifier library for code, infra, data, content, and media. |
| 9. Hardened Runner Backends | Secure local and remote runners with resource controls. |
| 10. Governance v2 | Central lock/policy bundles and auditable approvals. |
| 11. Hosted / Team Surfaces | Shared server, dashboard, approval inbox, runner and policy management. |
| 12. Analytics History | Trend intelligence and external exports. |
| 13. Specialized Domain Runtimes | Real runtime packs for code, infra, data, media, and research. |
| 14. Plugin Marketplace Governance | Test harness, scorecard, permissions, signing, identity, and review workflow. |

## Rules

- Work tasks in priority order unless a later task blocks the current task.
- Keep every Rust and JavaScript implementation file at or under 200 lines where practical.
- Update README and user docs in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Move task files to `done/` only after code, tests, documentation, and evidence are complete.
