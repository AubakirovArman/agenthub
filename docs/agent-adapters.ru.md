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

`command_template` больше не является user-facing provider field. AgentHub сам владеет API requests, logs, retries и будущими tool calls.

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

Non-project chat mode уже вызывает DeepSeek/Kimi напрямую со streaming output. Project transaction routes для `deepseek` и `kimi` используют AgentHub-owned API requests: provider возвращает JSON command plan, AgentHub валидирует и запускает эти команды внутри isolated worktree, затем продолжается обычный diff guard, verifier, rollback, commit и memory promotion flow.

Каждая transaction пишет выбранные routes в `.agent/tx/<tx-id>/agent_trace.json`. Adapter prompt artifacts пишутся как `.agent/tx/<tx-id>/agent_prompt_<role>.md`, а API executor plans/results пишутся как `.agent/tx/<tx-id>/api_execution_<role>.json`.
