# Remote Runner Execution

Языки: [English](remote-runner.en.md), [Русский](remote-runner.ru.md), [中文](remote-runner.zh.md), [Қазақша](remote-runner.kk.md)

Remote runner execution dispatches external agent adapter CLI calls, `execution.commands`, repair commands, review commands и verifier commands, когда `execution.sandbox.level` равен `2` или выше и настроен enterprise remote runner.

## Policy

```yaml
enterprise:
  runners:
    default: local
    remote:
      - id: strong-runner
        endpoint: ssh://runner.internal/workspaces/project
        labels:
          - strong-isolation
```

Поддерживаемые endpoints:

- `local://name`: dispatch path для local integration tests и single-host deployments.
- `ssh://host/path`: запускает `ssh host 'cd path && sh -lc <command>'`.
- `docker://image`: запускает `docker run --rm -i -v <worktree>:/workspace -w /workspace image sh -lc <command>`.

Docker runners optional. Нужен рабочий Docker-compatible CLI на host. Resource env vars применяются к container run:

```bash
AGENTHUB_CPU_CORES=2 AGENTHUB_MEMORY_MB=2048 AGENTHUB_NETWORK_MODE=none agenthub run task.yaml
```

## AgentSpec

```yaml
execution:
  sandbox:
    level: 2
  commands:
    - cargo test
```

Результаты собираются в `adapter_invocation_*.json`, `execution.json`, `verifier.json`, `review.json` или `repair.json` с `remote: true` и runner id. `sandbox.json` записывает выбранный runner.
