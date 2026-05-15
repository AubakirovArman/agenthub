# Remote Runner Execution

Languages: [English](remote-runner.en.md), [Русский](remote-runner.ru.md), [中文](remote-runner.zh.md), [Қазақша](remote-runner.kk.md)

Remote runner execution dispatches external agent adapter CLI calls, `execution.commands`, repair commands, review commands, and verifier commands when `execution.sandbox.level` is `2` or higher and an enterprise remote runner is configured.

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

Supported endpoints:

- `local://name`: dispatch path for local integration tests and single-host deployments.
- `ssh://host/path`: runs `ssh host 'cd path && sh -lc <command>'`.
- `docker://image`: runs `docker run --rm -i -v <worktree>:/workspace -w /workspace image sh -lc <command>`.

Docker runners are optional. They require a working Docker-compatible CLI on the host and inherit AgentHub resource environment variables:

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

Results are collected in `adapter_invocation_*.json`, `execution.json`, `verifier.json`, `review.json`, or `repair.json` with `remote: true` and the runner id. `sandbox.json` records the selected runner.
