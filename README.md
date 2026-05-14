# AgentHub

AgentHub is a transactional runtime foundation for AI-agent work.

The first implementation slice follows Phase 1 from `prd.md`: CLI skeleton,
transaction journal, isolated code workspace, verifier execution, diff guard,
rollback, sync check, and transaction reports.

## Current CLI

```bash
agenthub init
agenthub ask "Добавь страницу курсов в стиле dashboard"
agenthub run examples/command-task.yaml
agenthub run examples/content-task.yaml
agenthub tx status
agenthub tx report tx-...
agenthub workspace scan --write-maps
agenthub memory inspect
agenthub skills list
agenthub agents list
```

## Implemented Foundation

This repository currently implements the foundation slices from the PRD:

- transactional execution kernel;
- worktree-isolated `CodeWorkspace`, `ContentWorkspace`, `DataWorkspace`, and `InfraWorkspace` profiles;
- journal/report artifacts;
- diff guard and sync check;
- verifier commands and runtime smoke checks;
- bounded repair loop;
- VCM staging, promotion, failed attempts, and compacted state;
- observability artifacts, context pack trace, redaction, and cost placeholder;
- AgentSpec YAML, AgentIR, and execution DAG;
- skill manifests and dependency loading;
- agent adapter routing/traces;
- `single_executor` and `executor_reviewer_repair` topologies;
- reviewer gate with bounded repair before verifier;
- context maps for routes/components/exports;
- heuristic `ask` command for AgentSpec preview.

## Reviewer Topology

```yaml
topology:
  kind: executor_reviewer_repair

review:
  commands:
    - cargo test

repair:
  commands:
    - cargo fmt

transaction:
  max_repair_attempts: 1
```

## Local Build

Rust is required:

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```
