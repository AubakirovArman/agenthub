# PRD v2 Task 05 — Workspace Runtime Trait

Status: Done

## Goal

Move workspace execution behind a real `WorkspaceRuntime` abstraction so transaction code can work with pluggable domain runtimes instead of hard-coding the current Git worktree profile.

## Acceptance

- Define a `WorkspaceRuntime` trait for prepare, snapshot, run, diff, verify, commit, rollback, and cleanup responsibilities where supported by the current kernel.
- Extract the existing Git worktree behavior into a `CodeGitWorkspace` implementation.
- Keep transaction manager behavior compatible with existing `code.git` plans.
- Add structured runtime metadata to transaction artifacts or reports.
- Leave clear extension points for content, data, infra, media, and research runtimes.
- Tests prove existing code-git transactions still commit and roll back through the runtime path.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added `WorkspaceRuntime` with prepare, snapshot, run, diff, verify, commit, rollback, and cleanup methods.
- Added `CodeGitWorkspace` and moved Git worktree prepare/commit/rollback behavior behind the runtime implementation.
- Routed transaction prepare, commit, rollback, and cleanup through the runtime path while preserving existing `*.git` AgentSpec compatibility.
- Added `.agent/tx/<tx-id>/workspace_runtime.json` and a `Workspace Runtime` report section.
- Added runtime metadata assertions to commit and rollback transaction tests.
- Updated README and workspace runtime docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `f704db8`.
- Checks: `cargo fmt -- --check`; `scripts/check-module-size.sh 200`; `git diff --check`; `cargo test successful_transaction_commits_and_promotes_memory`; `cargo test failed_transaction_rolls_back_and_records_failed_attempt`; `cargo clippy -- -D warnings`; `cargo test`; `npm run check` in `editors/vscode`.
