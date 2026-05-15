# AgentHub Agent Topologies

Языки: [English](topologies.en.md), [Русский](topologies.ru.md), [中文](topologies.zh.md), [Қазақша](topologies.kk.md)

## Назначение

Topologies описывают, какие agent roles участвуют в transaction DAG. Phase 10 добавляет multi-role DAG planning, route traces для всех topology roles, model-call metadata для этих ролей и cost-aware routing metadata.

## Поддерживаемые kinds

- `single_executor`: одна executor role.
- `planner_executor`: planner затем executor.
- `executor_reviewer_repair`: executor, diff guard, reviewer, optional repair.
- `generator_critic`: generator, critic, executor.
- `swarm_research`: `researcher_1..N`, aggregator, executor.
- `manager_worker`: manager распределяет работу на `worker_1..N`, затем executor применяет managed result.
- `tournament`: candidates `contestant_1..N` идут в `judge`, затем executor применяет winning result.

Runtime mutation остаётся под контролем transaction kernel. Executor commands меняют workspace; reviewer и repair gates выполняются там, где они поддержаны. Остальные roles планируются, маршрутизируются, трассируются и попадают в DAG/gateway metadata.

## Planner / Executor Example

```bash
agenthub run examples/topology-planner-task.yaml
```

Ключевые поля AgentSpec:

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

- `.agent/tx/<tx-id>/dag.json`: содержит nodes `planner` и `executor`.
- `.agent/tx/<tx-id>/agent_trace.json`: содержит route каждой role.
- `.agent/tx/<tx-id>/model_call_metadata.json`: содержит planned calls для topology roles.

## Swarm Research Example

```bash
agenthub run examples/topology-swarm-task.yaml
```

```yaml
topology:
  kind: swarm_research
  swarm_size: 3
```

Это создаёт DAG roles `researcher_1`, `researcher_2`, `researcher_3`, `aggregator` и `executor`.

## Manager / Worker Example

```bash
agenthub run examples/topology-manager-worker-task.yaml
```

```yaml
topology:
  kind: manager_worker
  swarm_size: 2
```

Это создаёт fan-out DAG: `manager -> worker_1`, `manager -> worker_2`, а каждый worker ведёт к `executor`. `swarm_size` задаёт количество workers.

## Tournament Example

```bash
agenthub run examples/topology-tournament-task.yaml
```

```yaml
topology:
  kind: tournament
  swarm_size: 3
```

Это создаёт fan-in DAG: `contestant_1`, `contestant_2` и `contestant_3` идут в `judge`; `judge` ведёт к `executor`. `swarm_size` задаёт количество contestants и ограничивается диапазоном 2..8.

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

Repair route независим от executor route, поэтому repair agent может быть дешевле, приватнее или специализированнее.
