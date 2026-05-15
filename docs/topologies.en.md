# AgentHub Agent Topologies

Languages: [English](topologies.en.md), [Русский](topologies.ru.md), [中文](topologies.zh.md), [Қазақша](topologies.kk.md)

## Purpose

Topologies describe which agent roles participate in a transaction DAG. Phase 10 adds multi-role DAG planning, route traces for all topology roles, model-call metadata for those roles, and cost-aware routing metadata.

## Supported Kinds

- `single_executor`: one executor role.
- `planner_executor`: planner then executor.
- `executor_reviewer_repair`: executor, diff guard, reviewer, optional repair.
- `generator_critic`: generator, critic, executor.
- `swarm_research`: `researcher_1..N`, aggregator, executor.
- `manager_worker`: manager fans out to `worker_1..N`, then executor applies the managed result.
- `tournament`: `contestant_1..N` candidates feed `judge`, then executor applies the winning result.

Runtime mutation remains controlled by the existing transaction kernel. Executor commands mutate the workspace; reviewer and repair gates run where supported. Other roles are planned, routed, traced, and included in the DAG/gateway metadata.

## Planner / Executor Example

```bash
agenthub run examples/topology-planner-task.yaml
```

Key AgentSpec fields:

```yaml
topology:
  kind: planner_executor
  routing:
    cost_aware: true

agents:
  planner:
    adapter: codex
    dry_run: true
  executor:
    adapter: command
```

Artifacts:

- `.agent/tx/<tx-id>/dag.json`: contains `planner` and `executor` nodes.
- `.agent/tx/<tx-id>/agent_trace.json`: contains every role route.
- `.agent/tx/<tx-id>/model_call_metadata.json`: contains planned calls for topology roles.

## Swarm Research Example

```bash
agenthub run examples/topology-swarm-task.yaml
```

```yaml
topology:
  kind: swarm_research
  swarm_size: 3
```

This produces `researcher_1`, `researcher_2`, `researcher_3`, `aggregator`, and `executor` DAG roles.

## Manager / Worker Example

```bash
agenthub run examples/topology-manager-worker-task.yaml
```

```yaml
topology:
  kind: manager_worker
  swarm_size: 2
```

This produces a fan-out DAG: `manager -> worker_1`, `manager -> worker_2`, and each worker feeds `executor`. `swarm_size` controls the worker count.

## Tournament Example

```bash
agenthub run examples/topology-tournament-task.yaml
```

```yaml
topology:
  kind: tournament
  swarm_size: 3
```

This produces a fan-in DAG: `contestant_1`, `contestant_2`, and `contestant_3` feed `judge`; `judge` feeds `executor`. `swarm_size` controls the contestant count and is clamped to 2..8.

## Different Repair Agent

```yaml
topology:
  kind: executor_reviewer_repair

agents:
  executor:
    adapter: codex
    dry_run: true
  repair:
    adapter: gemini
    dry_run: true
```

The repair route is independent from the executor route, so the repair agent can be cheaper, private, or specialized.
