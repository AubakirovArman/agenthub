# AgentHub Agent Adapters

Languages: [English](agent-adapters.en.md), [Русский](agent-adapters.ru.md), [中文](agent-adapters.zh.md), [Қазақша](agent-adapters.kk.md)

## Purpose

AgentHub v0.4 moves adapter work away from external AI CLIs. User-facing AI providers are API-native `deepseek` and `kimi`; `command` remains the built-in deterministic runner used by the transaction kernel and tests.

The executor adapter still runs before `execution.commands`. Diff guard, reviewer gate, verifier, rollback, commit, memory promotion, and reports continue to use the same transaction flow.

## AgentSpec Fields

```yaml
agent:
  adapter: deepseek
  model: deepseek-chat
  dry_run: true
```

- `adapter`: `command`, `deepseek`, or `kimi`.
- `model`: optional model label recorded in traces and API requests.
- `dry_run`: writes adapter artifacts without making a provider request.

`command_template` is no longer a user-facing provider field. AgentHub owns API requests, logs, retries, and native command-plan tool calls directly.

Role-specific adapters can be set under `agents`:

```yaml
agents:
  executor:
    adapter: deepseek
    dry_run: true
  reviewer:
    adapter: kimi
    dry_run: true
```

## Current Project Executor Status

Non-project chat mode can call DeepSeek/Kimi directly with streaming output. Project transaction routes for `deepseek` and `kimi` use AgentHub-owned API requests: the provider calls the native `agenthub_command_plan` tool or returns a JSON command-plan fallback, AgentHub validates and permission-checks those commands inside the isolated worktree, then the normal diff guard, verifier, rollback, commit, and memory promotion flow continues.

Every transaction records selected routes in `.agent/tx/<tx-id>/agent_trace.json`. Adapter prompt artifacts are written as `.agent/tx/<tx-id>/agent_prompt_<role>.md`, native tool-loop receipts are written as `.agent/tx/<tx-id>/tool_loop_<role>.json`, and API executor plans/results are written as `.agent/tx/<tx-id>/api_execution_<role>.json`.
