# Phase 4 — AgentSpec YAML and Compiler

Status: Done

Closing evidence: runtime foundation plus compiler/spec tests.

## Deliverables

- AgentSpec YAML schema: done through Rust structs.
- Parser: done.
- Policy validator: done.
- Compiler to Execution DAG: done.
- AgentIR text form: done.
- Basic rules: done.

## Acceptance

- User can run `agenthub run task.yaml`: done.
- DAG generated from spec: done.
- Invalid scopes rejected before execution: done.

## Verification

- `src/spec/*`
- `src/compiler.rs`
- `cargo test`
