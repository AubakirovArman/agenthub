# AgentHub Testing Strategy

AgentHub 1.0 depends on trust: every transaction must either produce a verified commit, pause with clear human action, or roll back without contaminating the project. Testing is therefore a product surface, not only an engineering task.

## Test Pyramid

The required pyramid is:

```text
unit tests
integration tests
transaction scenario tests
fixture tests
dogfood tests
release smoke tests
```

Unit tests cover pure modules such as command policy, rollback handler selection, effect ledger records, AAL diagnostics, memory retrieval, provider metadata, and verifier parsing.

Integration tests cover real temp Git repositories and the transaction kernel. They must assert project state, transaction artifacts, memory state, reports, effects, and journal state.

Fixture tests run representative project profiles such as Rust, Python data, Terraform, content, media, research, and reference web apps.

Dogfood tests run real providers through AgentHub and record provider metrics, rollback behavior, and human-readable reports.

Release smoke tests prove that the installed binary can initialize a project, run doctor, inspect providers, execute a safe transaction, and generate a dashboard.

## P0 Transaction Scenarios

These scenarios are release gates:

- Success transaction: tx dir, worktree, command execution, diff guard, verifier, commit, memory promotion, report, WAL close, cleanup.
- Diff guard rollback: out-of-scope changes do not reach main, failed attempt is recorded, memory staging is not promoted.
- Verifier rollback: allowed changes are rolled back when verifier fails, report explains the verifier failure, memory is not promoted.
- No-commit mode: verifier can pass, status is `NOOP`, main remains unchanged, and memory is not promoted as project truth.
- Blocked-on-human: approval, missing environment, sync overlap, and missing runner cases pause without writing ordinary failed memory.
- Smart sync clean/rebase/overlap: independent main changes rebase and verify again; overlapping changes block.
- Memory promotion: only committed success promotes memory; rollback, noop, and blocked states do not.
- Effect ledger: planned, applied, verified, rollback, and non-rollbackable effects are written with handlers or explicit reasons.

## Runtime Reliability Scenarios

AgentHub must handle large or stuck processes:

- command prints large stdout;
- command prints large stderr;
- command produces infinite output;
- command hangs with no output;
- command exceeds timeout;
- process tree survives after parent exit.

The required behavior is bounded memory, process termination, log files under `.agent/tx/<tx-id>/logs/`, tails in JSON/report output, heartbeat events, and recoverable transaction state.

## Chaos Scenarios

Fault injection must eventually cover:

```text
WORKSPACE_READY
EXECUTING
DIFF_GUARD
VERIFYING
BEFORE_COMMIT
MEMORY_PROMOTION
CLEANUP
```

For each point, main must stay clean, memory must stay truthful, the journal must explain the state, and the transaction must be inspectable.

## Current Coverage

The Rust integration suite already covers the main transaction kernel, rollback, blocked approval, resume, smart sync rebase/overlap, sandbox levels, remote runner dispatch, repair, adaptive orchestration, and domain profiles. New 1.0 work should extend this suite before adding product UX around it.
