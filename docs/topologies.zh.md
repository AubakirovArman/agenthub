# AgentHub Agent Topologies

语言: [English](topologies.en.md), [Русский](topologies.ru.md), [中文](topologies.zh.md), [Қазақша](topologies.kk.md)

## 目的

Topologies 描述哪些 agent roles 参与 transaction DAG。Phase 10 添加 multi-role DAG planning、所有 topology roles 的 route traces、这些 roles 的 model-call metadata，以及 cost-aware routing metadata。

## 支持的 kinds

- `single_executor`: 一个 executor role。
- `planner_executor`: planner 然后 executor。
- `executor_reviewer_repair`: executor、diff guard、reviewer、optional repair。
- `generator_critic`: generator、critic、executor。
- `swarm_research`: `researcher_1..N`、aggregator、executor。
- `manager_worker`: manager 分发到 `worker_1..N`，然后 executor 应用 managed result。
- `tournament`: `contestant_1..N` candidates 输入 `judge`，然后 executor 应用 winning result。

Runtime mutation 仍由 transaction kernel 控制。Executor commands 修改 workspace；reviewer 和 repair gates 在支持的拓扑中运行。其他 roles 会被规划、路由、记录 trace，并进入 DAG/gateway metadata。

## Planner / Executor 示例

```bash
agenthub run examples/topology-planner-task.yaml
```

关键 AgentSpec 字段：

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

Artifacts：

- `.agent/tx/<tx-id>/dag.json`: 包含 `planner` 和 `executor` nodes。
- `.agent/tx/<tx-id>/agent_trace.json`: 包含每个 role route。
- `.agent/tx/<tx-id>/model_call_metadata.json`: 包含 topology roles 的 planned calls。

## Swarm Research 示例

```bash
agenthub run examples/topology-swarm-task.yaml
```

```yaml
topology:
  kind: swarm_research
  swarm_size: 3
```

这会生成 `researcher_1`、`researcher_2`、`researcher_3`、`aggregator` 和 `executor` DAG roles。

## Manager / Worker 示例

```bash
agenthub run examples/topology-manager-worker-task.yaml
```

```yaml
topology:
  kind: manager_worker
  swarm_size: 2
```

这会生成 fan-out DAG：`manager -> worker_1`、`manager -> worker_2`，每个 worker 都连接到 `executor`。`swarm_size` 控制 worker 数量。

## Tournament 示例

```bash
agenthub run examples/topology-tournament-task.yaml
```

```yaml
topology:
  kind: tournament
  swarm_size: 3
```

这会生成 fan-in DAG：`contestant_1`、`contestant_2` 和 `contestant_3` 输入 `judge`；`judge` 连接到 `executor`。`swarm_size` 控制 contestant 数量，并被限制在 2..8。

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

Repair route 独立于 executor route，因此 repair agent 可以更便宜、更私有或更专业。
