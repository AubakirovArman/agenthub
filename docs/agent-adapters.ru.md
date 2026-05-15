# AgentHub Agent Adapters

Языки: [English](agent-adapters.en.md), [Русский](agent-adapters.ru.md), [中文](agent-adapters.zh.md), [Қазақша](agent-adapters.kk.md)

## Назначение

Agent adapters позволяют одному и тому же transaction kernel запускать executor через локальные команды или внешний CLI-агент. В Phase 6 v1 поддерживаются `command`, `codex`, `kimi` и `gemini`.

Executor adapter запускается до `execution.commands`. Детерминированные команды, diff guard, reviewer gate, verifier, rollback, commit, memory promotion и reports остаются в том же транзакционном потоке.

## Поля AgentSpec

```yaml
agent:
  adapter: codex
  model: gpt-5.2
  dry_run: true
  command_template: "codex exec --sandbox workspace-write - < {prompt}"
```

- `adapter`: `command`, `codex`, `kimi` или `gemini`.
- `model`: optional model label; он пишется в traces и может использоваться в template.
- `dry_run`: пишет artifacts adapter-а, но не запускает внешний CLI.
- `command_template`: shell command для external CLI invocation.

Поддерживаемые placeholders:

- `{prompt}`: путь к `.agent/tx/<tx-id>/agent_prompt_executor.md`.
- `{role}`: текущая роль, обычно `executor`.
- `{model}`: настроенная модель или пустая строка.

Role-specific adapters можно задать в `agents`:

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

Role adapters запускаются перед deterministic commands своей роли: executor перед `execution.commands`, reviewer перед `review.commands`, repair перед каждой попыткой `repair.commands`.

## Пример

```bash
agenthub run examples/adapter-dry-run-task.yaml
```

Запуск того же spec через другой executor adapter без изменения YAML:

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

`AGENTHUB_PRIVATE_MODE=1` принудительно переключает маршрут на локальный `command` adapter и записывает причину fallback.

## Artifacts

Каждая транзакция пишет выбранные routes в `.agent/tx/<tx-id>/agent_trace.json`.

External role adapters также пишут:

- `.agent/tx/<tx-id>/agent_prompt_<role>.md`: redacted prompt, переданный CLI adapter-у.
- `.agent/tx/<tx-id>/adapter_invocation_<role>.json`: rendered command, exit code, redacted stdout/stderr, duration и dry-run flag.
- `.agent/tx/<tx-id>/agent_transcript.jsonl`: событие adapter run и затем command events.

Reviewer и repair roles в v1 маршрутизируются, вызываются при настройке и попадают в trace. Их deterministic `review.commands` и `repair.commands` продолжают выполняться через transaction kernel после adapter invocation.
