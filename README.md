# AgentHub

AgentHub is a transactional runtime foundation for AI-agent work.

The first implementation slice follows Phase 1 from `prd.md`: CLI skeleton,
transaction journal, isolated code workspace, verifier execution, diff guard,
rollback, sync check, and transaction reports.

## Current CLI

```bash
agenthub init
agenthub run examples/command-task.yaml
agenthub tx status
agenthub tx report tx-...
agenthub workspace scan
agenthub memory inspect
```

## Phase 1 Scope

This repository currently implements the execution kernel without an LLM
adapter. A transaction can run deterministic shell commands from an AgentSpec,
then verify and either merge or rollback the isolated worktree.

Later phases will add context packs, VCM-OS memory retrieval, skills, and agent
adapters.

## Local Build

Rust is required:

```bash
cargo build
cargo test
```

