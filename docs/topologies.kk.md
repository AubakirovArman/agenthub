# AgentHub Agent Topologies

Тілдер: [English](topologies.en.md), [Русский](topologies.ru.md), [中文](topologies.zh.md), [Қазақша](topologies.kk.md)

## Мақсаты

Topologies transaction DAG ішінде қандай agent roles қатысатынын сипаттайды. Phase 10 multi-role DAG planning, барлық topology roles үшін route traces, model-call metadata және cost-aware routing metadata қосады.

## Қолдау бар kinds

- `single_executor`: бір executor role.
- `planner_executor`: planner, содан кейін executor.
- `executor_reviewer_repair`: executor, diff guard, reviewer, optional repair.
- `generator_critic`: generator, critic, executor.
- `swarm_research`: `researcher_1..N`, aggregator, executor.
- `manager_worker`: manager `worker_1..N` roles ішіне таратады, содан кейін executor managed result қолданады.
- `tournament`: `contestant_1..N` candidates `judge` ішіне өтеді, содан кейін executor winning result қолданады.

Runtime mutation transaction kernel бақылауында қалады. Executor commands workspace өзгертеді; reviewer және repair gates қолдау бар topology ішінде орындалады. Басқа roles planned, routed, traced болады және DAG/gateway metadata ішіне кіреді.

## Planner / Executor Example

```bash
agenthub run examples/topology-planner-task.yaml
```

Негізгі AgentSpec fields:

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

- `.agent/tx/<tx-id>/dag.json`: `planner` және `executor` nodes қамтиды.
- `.agent/tx/<tx-id>/agent_trace.json`: әр role route қамтиды.
- `.agent/tx/<tx-id>/model_call_metadata.json`: topology roles planned calls қамтиды.

## Swarm Research Example

```bash
agenthub run examples/topology-swarm-task.yaml
```

```yaml
topology:
  kind: swarm_research
  swarm_size: 3
```

Бұл `researcher_1`, `researcher_2`, `researcher_3`, `aggregator` және `executor` DAG roles жасайды.

## Manager / Worker Example

```bash
agenthub run examples/topology-manager-worker-task.yaml
```

```yaml
topology:
  kind: manager_worker
  swarm_size: 2
```

Бұл fan-out DAG жасайды: `manager -> worker_1`, `manager -> worker_2`, және әр worker `executor` ішіне өтеді. `swarm_size` worker санын басқарады.

## Tournament Example

```bash
agenthub run examples/topology-tournament-task.yaml
```

```yaml
topology:
  kind: tournament
  swarm_size: 3
```

Бұл fan-in DAG жасайды: `contestant_1`, `contestant_2` және `contestant_3` `judge` ішіне өтеді; `judge` `executor` ішіне өтеді. `swarm_size` contestant санын басқарады және 2..8 диапазонына шектеледі.

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

Repair route executor route-тан тәуелсіз, сондықтан repair agent арзанырақ, private немесе specialized болуы мүмкін.
