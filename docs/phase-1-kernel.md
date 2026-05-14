# Phase 1 Kernel Design

## Goal

Build the smallest useful transaction kernel for code projects:

```text
AgentSpec -> journal -> isolated worktree -> execution -> diff guard -> verifier -> sync check -> commit or rollback -> report
```

## Non-Goals

- No LLM adapter yet.
- No custom AAL syntax yet.
- No full VCM-OS retrieval yet.
- No container sandbox yet.

## Transaction States

```text
CREATED
PREFLIGHT_CHECK
BASELINE_CAPTURED
WORKSPACE_READY
CONTEXT_PACK_BUILT
EXECUTING
DIFF_GUARD
VERIFYING
SYNC_CHECK
COMMITTING
COMMITTED
ROLLING_BACK
ROLLED_BACK
BLOCKED_ON_HUMAN
CLOSED
```

## AgentSpec v0

AgentSpec starts as YAML. The kernel supports:

- task metadata;
- `code.git` workspace;
- `git_worktree` isolation;
- allow/deny scope globs;
- deterministic execution commands;
- verifier commands;
- basic diff limits.

## Source of Truth

Transaction artifacts are stored under:

```text
.agent/tx/<tx-id>/
```

The append-only journal is the transaction truth. Reports are summaries for
humans.

