# Hardened Runner

Языки: [English](hardened-runner.en.md), [Русский](hardened-runner.ru.md), [中文](hardened-runner.zh.md), [Қазақша](hardened-runner.kk.md)

Hardened Runner показывает, как AgentHub выполняет команды локально или через remote runners. Это ещё не full kernel sandbox; слой делает runner trust, resource policy, process control и cancellation видимыми в transaction artifacts.

## Артефакты

Каждая транзакция пишет:

```text
.agent/tx/<tx-id>/runner.json
.agent/tx/<tx-id>/cancel_status.json
.agent/tx/<tx-id>/heartbeat.jsonl
```

Команды в `execution.json`, `review.json`, `repair.json` и `verifier.json` также содержат `runner_metadata` и `resource_usage`.

## Resource Policy

`runner.json` записывает timeout, CPU, memory, disk, network и filesystem policy. Текущий local execution реально применяет timeout и process-tree cleanup. Docker remote runners применяют CPU, memory и network options, если заданы `AGENTHUB_CPU_CORES`, `AGENTHUB_MEMORY_MB` и `AGENTHUB_NETWORK_MODE`. Disk пока остаётся recorded policy slot для будущих hardened backends.

## Cancellation

Используй CLI, чтобы запросить cancellation:

```bash
agenthub tx cancel tx-20260515123000-abcd1234 --reason "stop before deploy step"
```

Local runner проверяет cancel marker во время выполнения command, останавливает process tree, откатывает worktree, пишет `CANCELED` и не promotes memory. Также можно создать файл напрямую:

```text
.agent/tx/<tx-id>/cancel_request.json
```

Пример:

```json
{
  "requested_by": "operator",
  "reason": "stop before deploy step"
}
```

AgentHub пишет результат в `cancel_status.json`.

## Heartbeat

Долгие logged commands дописывают heartbeat records:

```json
{"event":"HEARTBEAT","node":"verifier-0","elapsed_sec":30,"last_output_sec":5}
```

Интервал по умолчанию 30 секунд. В тестах его можно уменьшить через `AGENTHUB_HEARTBEAT_INTERVAL_MS`.
