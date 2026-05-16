# AgentHub Agent Adapters

Языки: [English](agent-adapters.en.md), [Русский](agent-adapters.ru.md), [中文](agent-adapters.zh.md), [Қазақша](agent-adapters.kk.md)

## Назначение

В AgentHub v0.4 adapter layer уходит от внешних AI CLI. User-facing AI providers теперь API-native `deepseek` и `kimi`; `command` остается встроенным deterministic runner для transaction kernel и тестов.

Executor adapter по-прежнему запускается перед `execution.commands`. Diff guard, reviewer gate, verifier, rollback, commit, memory promotion и reports используют тот же transaction flow.

## AgentSpec fields

```yaml
agent:
  adapter: deepseek
  model: deepseek-chat
  dry_run: true
```

- `adapter`: `command`, `deepseek` или `kimi`.
- `model`: optional model label, который пишется в traces и API requests.
- `dry_run`: пишет adapter artifacts без provider request.

`command_template` больше не является user-facing provider field. AgentHub сам владеет API requests, logs, retries и native command-plan tool calls.

Role-specific adapters можно задавать в `agents`:

```yaml
agents:
  executor:
    adapter: deepseek
    dry_run: true
  reviewer:
    adapter: kimi
    dry_run: true
```

## Текущий статус project executor

Non-project chat mode уже вызывает DeepSeek/Kimi напрямую со streaming output. Project transaction routes для `deepseek` и `kimi` используют AgentHub-owned API requests: provider может вызвать bounded builtin read/search/read-only-shell tools для контекста, AgentHub reinjects redacted tool results в тот же turn, затем provider вызывает native `agenthub_command_plan` tool call или возвращает JSON command plan fallback. AgentHub валидирует и permission-checks эти команды внутри isolated worktree, затем продолжается обычный diff guard, verifier, rollback, commit и memory promotion flow.

Каждая transaction пишет выбранные routes в `.agent/tx/<tx-id>/agent_trace.json`. Adapter prompt artifacts пишутся как `.agent/tx/<tx-id>/agent_prompt_<role>.md`, native command-plan tool-loop receipts пишутся как `.agent/tx/<tx-id>/tool_loop_<role>.json`, builtin tool-result reinjection receipts пишутся как `.agent/tx/<tx-id>/tool_results_<role>.json`, а API executor plans/results пишутся как `.agent/tx/<tx-id>/api_execution_<role>.json`.
