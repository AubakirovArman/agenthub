# Remote Runner Execution

Тілдер: [English](remote-runner.en.md), [Русский](remote-runner.ru.md), [中文](remote-runner.zh.md), [Қазақша](remote-runner.kk.md)

`execution.sandbox.level` `2` немесе одан жоғары болса және enterprise remote runner бапталса, Remote runner execution external agent adapter CLI calls, `execution.commands`, repair commands, review commands және verifier commands dispatch жасайды.

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

Қолдау бар endpoints:

- `local://name`: local integration tests және single-host deployments үшін dispatch path.
- `ssh://host/path`: `ssh host 'cd path && sh -lc <command>'` орындайды.
- `docker://image`: `docker run --rm -i -v <worktree>:/workspace -w /workspace image sh -lc <command>` орындайды.

Docker runners optional. Host ішінде Docker-compatible CLI жұмыс істеуі керек. Resource env vars container run үшін қолданылады:

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

Нәтижелер `adapter_invocation_*.json`, `execution.json`, `verifier.json`, `review.json` немесе `repair.json` ішіне `remote: true` және runner id арқылы жиналады. `sandbox.json` таңдалған runner жазады.
