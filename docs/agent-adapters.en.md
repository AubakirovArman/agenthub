# AgentHub Agent Adapters

Languages: [English](agent-adapters.en.md), [Русский](agent-adapters.ru.md), [中文](agent-adapters.zh.md), [Қазақша](agent-adapters.kk.md)

## Purpose

Agent adapters let the same transaction kernel route executor work through local commands or an external CLI agent. Phase 6 v1 supports `command`, `codex`, `kimi`, and `gemini`.

The executor adapter runs before `execution.commands`. Deterministic commands, diff guard, reviewer gate, verifier, rollback, commit, memory promotion, and reports still use the same transaction flow.

## AgentSpec Fields

```yaml
agent:
  adapter: codex
  model: gpt-5.2
  dry_run: true
  command_template: "codex exec --sandbox workspace-write - < {prompt}"
```

- `adapter`: `command`, `codex`, `kimi`, or `gemini`.
- `model`: optional model label recorded in traces and usable in templates.
- `dry_run`: writes adapter artifacts without executing the external CLI.
- `command_template`: shell command used for external CLI invocation.

Supported template placeholders:

- `{prompt}`: path to `.agent/tx/<tx-id>/agent_prompt_executor.md`.
- `{role}`: current role, usually `executor`.
- `{model}`: configured model or an empty string.

Role-specific adapters can be set under `agents`:

```yaml
agents:
  executor:
    adapter: codex
    dry_run: true
  reviewer:
    adapter: gemini
    dry_run: true
  repair:
    adapter: kimi
    dry_run: true
```

Role adapters run before that role's deterministic commands: executor before `execution.commands`, reviewer before `review.commands`, and repair before each `repair.commands` attempt.

## Example

```bash
agenthub run examples/adapter-dry-run-task.yaml
```

To run the same spec through another executor adapter without editing YAML:

```bash
AGENTHUB_EXECUTOR_ADAPTER=kimi AGENTHUB_ADAPTER_DRY_RUN=1 agenthub run examples/adapter-dry-run-task.yaml
```

## Environment Overrides

```bash
AGENTHUB_EXECUTOR_ADAPTER=gemini
AGENTHUB_AGENT_ADAPTER=codex
AGENTHUB_ADAPTER_DRY_RUN=1
AGENTHUB_ADAPTER_CODEX_TEMPLATE='codex exec --sandbox workspace-write - < {prompt}'
AGENTHUB_ADAPTER_KIMI_TEMPLATE='kimi --print --afk --input-format text < {prompt}'
AGENTHUB_ADAPTER_GEMINI_TEMPLATE='gemini --prompt-file {prompt}'
AGENTHUB_ADAPTER_CODEX_MODEL='gpt-5.2'
AGENTHUB_PRIVATE_MODE=1
```

`AGENTHUB_PRIVATE_MODE=1` forces fallback to the local `command` adapter and records the fallback reason.

## Artifacts

Every transaction records selected routes in `.agent/tx/<tx-id>/agent_trace.json`.

External role adapters also write:

- `.agent/tx/<tx-id>/agent_prompt_<role>.md`: redacted prompt sent to the CLI adapter.
- `.agent/tx/<tx-id>/adapter_invocation_<role>.json`: rendered command, exit code, redacted stdout/stderr, duration, and dry-run flag.
- `.agent/tx/<tx-id>/agent_transcript.jsonl`: adapter run event followed by command events.

Reviewer and repair roles are routed, invoked when configured, and traced in v1. Their deterministic `review.commands` and `repair.commands` continue to run through the transaction kernel after adapter invocation.
