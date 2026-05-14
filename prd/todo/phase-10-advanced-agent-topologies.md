# Phase 10 — Advanced Agent Topologies

Status: Todo

Order: current next phase.

## Deliverables

- Planner/executor.
- Executor/reviewer/repair.
- Generator/critic.
- Swarm research.
- Cost-aware routing.

## Acceptance

- DAG can contain multiple model roles.
- Reviewer can block bad output.
- Repair agent can be different from executor.

## Implementation Notes

- Existing `executor_reviewer_repair` must be preserved.
- Add DAG support for additional role nodes before adding runtime execution semantics.
- Add route metadata for cost-aware decisions without executing remote calls by default.
- Update docs on 4 languages before moving this file to `done/`.
